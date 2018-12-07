#![allow(dead_code)]

#[cfg(test)]
use crate::ast::{Meta, Type};

use crate::ast::{Arg, Expr, Module, Statement};
use crate::pretty::*;

use heck::{CamelCase, SnakeCase};
use itertools::Itertools;
use std::char;

const INDENT: isize = 4;

#[derive(Debug, Clone, Default)]
struct ExprEnv {}

pub fn module<T>(module: Module<T>) -> String {
    format!("-module({}).", module.name)
        .to_doc()
        .append(line())
        .append(
            module
                .statements
                .into_iter()
                .map(statement)
                .collect::<Vec<_>>(),
        )
        .format(80)
}

fn statement<T>(statement: Statement<T>) -> Document {
    match statement {
        Statement::Test { .. } => unimplemented!(),
        Statement::Enum { .. } => nil(),
        Statement::Import { .. } => nil(),
        Statement::ExternalType { .. } => nil(),
        Statement::Fun {
            args,
            public,
            name,
            body,
            ..
        } => mod_fun(public, name, args, body),
        Statement::ExternalFun {
            fun,
            module,
            args,
            public,
            name,
            ..
        } => external_fun(public, name, module, fun, args.len()),
    }
}

fn mod_fun<T>(public: bool, name: String, args: Vec<Arg>, body: Expr<T>) -> Document {
    let args_doc = arg_list(
        args.iter()
            .map(|a| a.name.to_camel_case().to_doc())
            .collect(),
    );
    export(public, &name, args.len())
        .append(line())
        .append(name.to_doc())
        .append(args_doc)
        .append(" ->".to_doc())
        .append(
            line()
                .append(expr(body, &mut ExprEnv::default()))
                .append(".")
                .nest(INDENT),
        )
        .append(line())
}

fn export(public: bool, name: &String, arity: usize) -> Document {
    if public {
        format!("-export([{}/{}]).", name, arity)
            .to_doc()
            .append(line())
    } else {
        nil()
    }
}

// TODO: Escape
fn atom(value: String) -> Document {
    value.to_doc().surround("'", "'")
}

fn expr<T>(expression: Expr<T>, mut env: &ExprEnv) -> Document {
    match expression {
        Expr::Int { value, .. } => value.to_doc(),
        Expr::Float { value, .. } => value.to_doc(),
        Expr::Atom { value, .. } => atom(value),
        Expr::String { .. } => unimplemented!(),
        Expr::Tuple { .. } => unimplemented!(),
        Expr::Seq { .. } => unimplemented!(),
        Expr::Var { .. } => unimplemented!(),
        Expr::Fun { .. } => unimplemented!(),
        Expr::Nil { .. } => "[]".to_doc(),
        Expr::Cons { .. } => unimplemented!(),
        Expr::Call { .. } => unimplemented!(),
        Expr::BinOp { .. } => unimplemented!(),
        Expr::Let { .. } => unimplemented!(),
        Expr::Enum { name, args, .. } => {
            if args.len() == 0 {
                atom(name.to_snake_case())
            } else {
                unimplemented!()

                // name.to_doc().append(
                //     args.into_iter()
                //         .map(|e| expr(e, &mut env))
                //         .intersperse(delim(";"))
                //         .collect::<Vec<_>>()
                //         .to_doc()
                //         .nest_current()
                //         .surround("(", ")"),
                // )
            }
        }
        Expr::Case { .. } => unimplemented!(),
        Expr::RecordNil { .. } => unimplemented!(),
        Expr::RecordCons { .. } => unimplemented!(),
        Expr::RecordSelect { .. } => unimplemented!(),
        Expr::ModuleSelect { .. } => unimplemented!(),
    }
}

fn arg_list(args: Vec<Document>) -> Document {
    args.into_iter()
        .intersperse(delim(","))
        .collect::<Vec<_>>()
        .to_doc()
        .nest_current()
        .surround("(", ")")
        .group()
}

fn external_fun(public: bool, name: String, module: String, fun: String, arity: usize) -> Document {
    let chars: String = (65..(65 + arity))
        .map(|x| x as u8 as char)
        .map(|c| c.to_string())
        .intersperse(", ".to_string())
        .collect();

    let header = format!("{}({}) ->", name, chars).to_doc();
    let body = format!("{}:{}({}).", module, fun, chars).to_doc();

    line()
        .to_doc()
        .append(export(public, &name, arity))
        .append(header)
        .append(line().append(body).nest(INDENT))
        .append(line())
}

#[test]
fn module_test() {
    let m: Module<()> = Module {
        name: "magic".to_string(),
        statements: vec![
            Statement::ExternalType {
                meta: Meta {},
                public: true,
                name: "Any".to_string(),
            },
            Statement::Enum {
                meta: Meta {},
                public: true,
                name: "Any".to_string(),
                args: vec![],
                constructors: vec![Type::Constructor {
                    meta: Meta {},
                    args: vec![],
                    name: "Ok".to_string(),
                }],
            },
            Statement::Import {
                meta: Meta {},
                module: "result".to_string(),
            },
            Statement::ExternalFun {
                meta: Meta {},
                args: vec![
                    Type::Constructor {
                        meta: Meta {},
                        args: vec![],
                        name: "Int".to_string(),
                    },
                    Type::Constructor {
                        meta: Meta {},
                        args: vec![],
                        name: "Int".to_string(),
                    },
                ],
                name: "add_ints".to_string(),
                fun: "add".to_string(),
                module: "int".to_string(),
                public: false,
                retrn: Type::Constructor {
                    meta: Meta {},
                    args: vec![],
                    name: "Int".to_string(),
                },
            },
            Statement::ExternalFun {
                meta: Meta {},
                args: vec![],
                name: "map".to_string(),
                fun: "new".to_string(),
                module: "maps".to_string(),
                public: true,
                retrn: Type::Constructor {
                    meta: Meta {},
                    args: vec![],
                    name: "Map".to_string(),
                },
            },
        ],
    };
    let expected = "-module(magic).

add_ints(A, B) ->
    int:add(A, B).

-export([map/0]).
map() ->
    maps:new().
"
    .to_string();
    assert_eq!(expected, module(m));
}

#[test]
fn expr_test() {
    let m: Module<()> = Module {
        name: "term".to_string(),
        statements: vec![
            Statement::Fun {
                meta: Meta {},
                public: false,
                args: vec![],
                name: "atom".to_string(),
                body: Expr::Atom {
                    meta: Meta {},
                    value: "ok".to_string(),
                },
            },
            Statement::Fun {
                meta: Meta {},
                public: false,
                args: vec![],
                name: "int".to_string(),
                body: Expr::Int {
                    meta: Meta {},
                    value: 176,
                },
            },
            Statement::Fun {
                meta: Meta {},
                public: false,
                args: vec![],
                name: "float".to_string(),
                body: Expr::Float {
                    meta: Meta {},
                    value: 11177.324401,
                },
            },
            Statement::Fun {
                meta: Meta {},
                public: false,
                args: vec![],
                name: "nil".to_string(),
                body: Expr::Nil {
                    meta: Meta {},
                    typ: (),
                },
            },
            Statement::Fun {
                meta: Meta {},
                public: false,
                args: vec![],
                name: "enum1".to_string(),
                body: Expr::Enum {
                    meta: Meta {},
                    name: "Nil".to_string(),
                    typ: (),
                    args: vec![],
                },
            },
            // Statement::Fun {
            //     meta: Meta {},
            //     public: false,
            //     args: vec![],
            //     name: "enum2".to_string(),
            //     body: Expr::Enum {
            //         meta: Meta {},
            //         name: "Ok".to_string(),
            //         typ: (),
            //         args: vec![
            //             Expr::Int {
            //                 meta: Meta {},
            //                 value: 1,
            //             },
            //             Expr::Float {
            //                 meta: Meta {},
            //                 value: 2.0,
            //             },
            //         ],
            //     },
            // },
        ],
    };
    let expected = "-module(term).

atom() ->
    'ok'.

int() ->
    176.

float() ->
    11177.324401.

nil() ->
    [].

enum1() ->
    'nil'.
"
    //
    // enum2() ->
    //     {'ok', 1, 2.0}.
    .to_string();
    assert_eq!(expected, module(m));
}

#[test]
fn args_test() {
    let m: Module<()> = Module {
        name: "term".to_string(),
        statements: vec![Statement::Fun {
            meta: Meta {},
            public: false,
            name: "some_function".to_string(),
            args: vec![
                Arg {
                    name: "arg_one".to_string(),
                },
                Arg {
                    name: "arg_two".to_string(),
                },
                Arg {
                    name: "arg_3".to_string(),
                },
                Arg {
                    name: "arg4".to_string(),
                },
                Arg {
                    name: "arg_four".to_string(),
                },
                Arg {
                    name: "arg__five".to_string(),
                },
                Arg {
                    name: "arg_six".to_string(),
                },
                Arg {
                    name: "arg_that_is_long".to_string(),
                },
            ],
            body: Expr::Atom {
                meta: Meta {},
                value: "ok".to_string(),
            },
        }],
    };
    let expected = "-module(term).

some_function(ArgOne,
              ArgTwo,
              Arg3,
              Arg4,
              ArgFour,
              ArgFive,
              ArgSix,
              ArgThatIsLong) ->
    'ok'.
"
    .to_string();
    assert_eq!(expected, module(m));
}
