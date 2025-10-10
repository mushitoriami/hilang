use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::process;

#[derive(Debug, PartialEq)]
enum AST {
    Scope(Box<AST>),
    Arrow(Box<AST>, Box<AST>),
    Match(Box<AST>, Box<AST>),
    Method(Vec<AST>, Box<AST>),
    Primitive(String),
    Literal(String),
    Variable(String),
}

fn parse_ast(input: &str) -> Result<AST, ()> {
    let mut tokenizer = kohaku::Tokenizer::new([
        ";", "|", "->", "<-", "=<", "==", "!=", "<", "+", "-", "*", "%", "\\", ".", "(", ")",
    ]);
    let mut parser = suzuran::Parser::new([
        ";", "|", "->", "<-", "=<", "==", "!=", "<", "+", "-", "*", "%", "\\", ".",
    ]);
    let iter = tokenizer.tokenize(input).map_while(|x| x.ok());
    let node = parser.parse(iter).ok_or(())?;
    convert(node)
}

fn convert(node: suzuran::Node) -> Result<AST, ()> {
    match node {
        suzuran::Node::Placeholder() => Err(()),
        suzuran::Node::Parentheses(n) => Ok(AST::Scope(Box::new(convert(*n)?))),
        suzuran::Node::Primitive(label) => match label.starts_with(r#"""#) {
            true => Ok(AST::Literal(label.trim_matches('"').to_string())),
            false => Ok(AST::Primitive(label)),
        },
        suzuran::Node::Operator(label, n1, n2) if label == "\\" => {
            if let suzuran::Node::Placeholder() = *n1
                && let suzuran::Node::Primitive(label) = *n2
            {
                Ok(AST::Variable(label))
            } else {
                Err(())
            }
        }
        suzuran::Node::Operator(label, n1, n2) => {
            let a1 = convert(*n1)?;
            let a2 = convert(*n2)?;
            match label.as_str() {
                ";" => Ok(AST::Arrow(Box::new(a1), Box::new(a2))),
                "->" => Ok(AST::Arrow(Box::new(a1), Box::new(a2))),
                "<-" => Ok(AST::Arrow(Box::new(a2), Box::new(a1))),
                "|" => Ok(AST::Match(Box::new(a1), Box::new(a2))),
                "." => Ok(AST::Method(vec![a1], Box::new(a2))),
                _ => Ok(AST::Method(vec![a1, a2], Box::new(AST::Primitive(label)))),
            }
        }
    }
}

struct Interpreter {
    storage: HashMap<String, DataInterpreter>,
}

impl Interpreter {
    fn new(storage: HashMap<String, DataInterpreter>) -> Self {
        Interpreter { storage }
    }

    fn interpret(
        &mut self,
        args: &[AST],
        ast: &AST,
        stream: DataInterpreter,
    ) -> Option<DataInterpreter> {
        match ast {
            AST::Arrow(obj1, obj2) => self
                .interpret(args, obj1, stream)
                .and_then(|stream| self.interpret(args, obj2, stream)),
            AST::Match(obj1, obj2) => self
                .interpret(args, obj1, stream.clone())
                .or_else(|| self.interpret(args, obj2, stream)),
            AST::Method(args, obj) => self.interpret(args, obj, stream),
            AST::Primitive(label) => self.interpret_primitive(args, label, stream),
            AST::Variable(label) => todo!(),
            AST::Literal(contents) => self.interpret_literal(args, contents, stream),
            AST::Scope(obj) => self.interpret(args, obj, stream),
        }
    }

    fn interpret_literal(
        &mut self,
        args: &[AST],
        contents: &str,
        stream: DataInterpreter,
    ) -> Option<DataInterpreter> {
        match stream == DataInterpreter::Void() && args.is_empty() {
            true => Some(DataInterpreter::Str(contents.to_string())),
            false => panic!(),
        }
    }

    fn interpret_primitive(
        &mut self,
        args: &[AST],
        label: &str,
        stream: DataInterpreter,
    ) -> Option<DataInterpreter> {
        match label {
            "int" => match stream {
                DataInterpreter::Int(i) => Some(DataInterpreter::Int(i)),
                DataInterpreter::Str(s) => Some(DataInterpreter::Int(s.parse::<i64>().ok()?)),
                DataInterpreter::Void() => panic!(),
            },
            "str" => match stream {
                DataInterpreter::Int(i) => Some(DataInterpreter::Str(i.to_string())),
                DataInterpreter::Str(s) => Some(DataInterpreter::Str(s)),
                DataInterpreter::Void() => panic!(),
            },
            "output" => {
                match stream {
                    DataInterpreter::Int(i) => println!("{}", i),
                    DataInterpreter::Str(s) => println!("{}", s),
                    DataInterpreter::Void() => panic!(),
                };
                Some(DataInterpreter::Void())
            }
            "input" => {
                if stream != DataInterpreter::Void() {
                    panic!()
                }
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                Some(DataInterpreter::Str(input.trim_end().to_string()))
            }
            "store" => {
                match self.interpret(&[], &args[0], DataInterpreter::Void()) {
                    Some(DataInterpreter::Str(label)) => self.storage.insert(label, stream),
                    _ => panic!(),
                };
                Some(DataInterpreter::Void())
            }
            "load" => {
                if stream != DataInterpreter::Void() {
                    panic!()
                }
                match self.interpret(&[], &args[0], DataInterpreter::Void()) {
                    Some(DataInterpreter::Str(label)) => self.storage.get(&label).cloned(),
                    _ => panic!(),
                }
            }
            "+" | "-" | "*" | "%" | "==" | "!=" | "=<" | "<" => {
                let o1 = self.interpret(&[], &args[0], DataInterpreter::Void())?;
                let o2 = self.interpret(&[], &args[1], DataInterpreter::Void())?;
                match (o1, o2) {
                    (DataInterpreter::Int(i1), DataInterpreter::Int(i2)) => match label {
                        "+" => Some(DataInterpreter::Int(i1 + i2)),
                        "-" => Some(DataInterpreter::Int(i1 - i2)),
                        "*" => Some(DataInterpreter::Int(i1 * i2)),
                        "%" => Some(DataInterpreter::Int(i1 % i2)),
                        "==" => (i1 == i2).then_some(DataInterpreter::Void()),
                        "!=" => (i1 != i2).then_some(DataInterpreter::Void()),
                        "=<" => (i1 <= i2).then_some(DataInterpreter::Void()),
                        "<" => (i1 < i2).then_some(DataInterpreter::Void()),
                        _ => panic!(),
                    },
                    _ => panic!(),
                }
            }
            "loop" => {
                let mut stream_loop = Some(stream);
                while let Some(stream) = stream_loop {
                    stream_loop = self.interpret(&[], &args[0], stream);
                }
                stream_loop
            }
            "pass" => Some(stream),
            _ => panic!(),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
enum DataInterpreter {
    Int(i64),
    Str(String),
    Void(),
}

fn main() {
    let args = env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        eprintln!("Usage: hilang <filename>");
        process::exit(1);
    }
    let Ok(mut file) = File::open(&args[1]) else {
        eprintln!("Cannot open file: {}", &args[1]);
        process::exit(1);
    };
    let mut contents = String::new();
    let Ok(_) = file.read_to_string(&mut contents) else {
        eprintln!("Cannot read file: {}", &args[1]);
        process::exit(1);
    };
    let Ok(ast) = parse_ast(&contents) else {
        eprintln!("Cannot parse file: {}", &args[1]);
        process::exit(1);
    };
    let mut interpreter = Interpreter::new(HashMap::new());
    let Some(DataInterpreter::Void()) = interpreter.interpret(&[], &ast, DataInterpreter::Void())
    else {
        eprintln!("Cannot execute successfully: {}", &args[1]);
        process::exit(1);
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_1() {
        assert_eq!(parse_ast("{aaa ->bbb }"), Err(()));
    }

    #[test]
    fn test_parse_2() {
        assert_eq!(
            parse_ast("(aaa ->bbb )"),
            Ok(AST::Scope(Box::new(AST::Arrow(
                Box::new(AST::Primitive("aaa".to_string())),
                Box::new(AST::Primitive("bbb".to_string()))
            ))))
        );
    }

    #[test]
    fn test_parse_3() {
        assert_eq!(
            parse_ast("{inst1 -> inst2 -> {inst4 <- inst3} -> inst5}"),
            Err(())
        );
    }

    #[test]
    fn test_parse_4() {
        assert_eq!(
            parse_ast("inst1 -> inst2 -> (inst4 <- inst3) -> inst5"),
            Ok(AST::Arrow(
                Box::new(AST::Arrow(
                    Box::new(AST::Arrow(
                        Box::new(AST::Primitive("inst1".to_string())),
                        Box::new(AST::Primitive("inst2".to_string()))
                    )),
                    Box::new(AST::Scope(Box::new(AST::Arrow(
                        Box::new(AST::Primitive("inst3".to_string())),
                        Box::new(AST::Primitive("inst4".to_string()))
                    ))))
                )),
                Box::new(AST::Primitive("inst5".to_string()))
            ))
        );
    }

    #[test]
    fn test_parse_5() {
        assert_eq!(
            parse_ast("{a=(P -> Q), b={c=(R -> {S <- T}), d={U <- V}}}"),
            Err(())
        );
    }

    #[test]
    fn test_parse_6() {
        assert_eq!(
            parse_ast("(a -> b | c -> d)"),
            Ok(AST::Scope(Box::new(AST::Match(
                Box::new(AST::Arrow(
                    Box::new(AST::Primitive("a".to_string())),
                    Box::new(AST::Primitive("b".to_string()))
                )),
                Box::new(AST::Arrow(
                    Box::new(AST::Primitive("c".to_string())),
                    Box::new(AST::Primitive("d".to_string()))
                ))
            ))))
        );
    }

    #[test]
    fn test_parse_7() {
        assert_eq!(
            parse_ast("((P -> Q)<-(R -> S)).a"),
            Ok(AST::Method(
                vec![AST::Scope(Box::new(AST::Arrow(
                    Box::new(AST::Scope(Box::new(AST::Arrow(
                        Box::new(AST::Primitive("R".to_string())),
                        Box::new(AST::Primitive("S".to_string()))
                    )))),
                    Box::new(AST::Scope(Box::new(AST::Arrow(
                        Box::new(AST::Primitive("P".to_string())),
                        Box::new(AST::Primitive("Q".to_string()))
                    ))))
                )))],
                Box::new(AST::Primitive("a".to_string()))
            ))
        );
    }

    #[test]
    fn test_parse_8() {
        assert_eq!(
            parse_ast(r#"("3".int -> push -> "2".int -> push -> add -> pop -> "i".write)"#),
            Ok(AST::Scope(Box::new(AST::Arrow(
                Box::new(AST::Arrow(
                    Box::new(AST::Arrow(
                        Box::new(AST::Arrow(
                            Box::new(AST::Arrow(
                                Box::new(AST::Arrow(
                                    Box::new(AST::Method(
                                        vec![AST::Literal("3".to_string())],
                                        Box::new(AST::Primitive("int".to_string()))
                                    )),
                                    Box::new(AST::Primitive("push".to_string()))
                                )),
                                Box::new(AST::Method(
                                    vec![AST::Literal("2".to_string())],
                                    Box::new(AST::Primitive("int".to_string()))
                                ))
                            )),
                            Box::new(AST::Primitive("push".to_string()))
                        )),
                        Box::new(AST::Primitive("add".to_string()))
                    )),
                    Box::new(AST::Primitive("pop".to_string()))
                )),
                Box::new(AST::Method(
                    vec![AST::Literal("i".to_string())],
                    Box::new(AST::Primitive("write".to_string()))
                ))
            ))))
        );
    }

    #[test]
    fn test_parse_9() {
        assert_eq!(parse_ast("#"), Err(()));
    }

    #[test]
    fn test_parse_10() {
        assert_eq!(parse_ast(r#"\abc"#), Ok(AST::Variable("abc".to_string())));
    }

    #[test]
    fn test_interpreter_1() {
        let program = r#"("3" -> int) + "a".load -> "b".store -> "b".load"#;
        let ast = parse_ast(program).unwrap();
        let mut interpreter =
            Interpreter::new(HashMap::from([("a".to_string(), DataInterpreter::Int(5))]));
        assert_eq!(
            interpreter.interpret(&[], &ast, DataInterpreter::Void()),
            Some(DataInterpreter::Int(8))
        );
        assert_eq!(
            interpreter.storage,
            HashMap::from([
                ("a".to_string(), DataInterpreter::Int(5)),
                ("b".to_string(), DataInterpreter::Int(8))
            ])
        );
    }

    #[test]
    fn test_interpreter_2() {
        let program = r#""x".store;
"1" -> int -> "i".store;
"0" -> int -> "r".store;
(
    "i".load =< "x".load;
    "2" -> int -> "j".store;
    (
        "j".load < "i".load -> ("i".load % "j".load) != ("0" -> int);
        "j".load + ("1" -> int) -> "j".store
    ).loop | pass;
    "j".load == "i".load -> "r".load + "i".load -> "r".store
        | pass;
    "i".load + ("1" -> int) -> "i".store
).loop | pass;
"r".load"#;
        let ast = parse_ast(program).unwrap();
        let mut interpreter = Interpreter::new(HashMap::new());
        assert_eq!(
            interpreter.interpret(&[], &ast, DataInterpreter::Int(30)),
            Some(DataInterpreter::Int(129))
        );
    }
}
