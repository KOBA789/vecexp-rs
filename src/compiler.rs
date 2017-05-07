use index::{FeatId};
use vm::InstCode;
use std::collections::LinkedList;
use combine::*;
use combine::char::*;
use combine::combinator::*;

#[derive(Debug)]
pub enum Node {
    Pattern(Vec<Option<FeatId>>),
    Union(Box<Node>, Box<Node>),
    Concat(Vec<Node>),
    Star(Box<Node>),
    Empty,
}

type FnPtrParser<O, I> = FnParser<I, fn(I) -> ParseResult<O, I>>;
type QueryParser<O, I> = Expected<FnPtrParser<O, I>>;

struct Query<I>(::std::marker::PhantomData<fn(I) -> I>);

fn fn_parser<O, I>(f: fn(I) -> ParseResult<O, I>, err: &'static str) -> QueryParser<O, I>
    where I: Stream<Item = char>
{
    parser(f).expected(err)
}

impl<I> Query<I>
    where I: Stream<Item = char>
{
    fn integer() -> QueryParser<u32, I> {
        fn_parser(Query::<I>::integer_, "integer")
    }
    fn integer_(input: I) -> ParseResult<u32, I> {
        many1::<String, _>(digit()).map(|ds| ds.parse::<u32>().unwrap()).parse_lazy(input).into()
    }

    fn pattern() -> QueryParser<Node, I> {
        fn_parser(Query::<I>::pattern_, "pattern")
    }
    fn pattern_(input: I) -> ParseResult<Node, I> {
        sep_by1(Query::<I>::integer().map(|a| Some(a)).or(char('*').map(|_| None)), char('-')).map(|feat_id| Node::Pattern(feat_id)).parse_lazy(input).into()
    }

    fn concat() -> QueryParser<Node, I> {
        fn_parser(Query::<I>::concat_, "concat")
    }
    fn concat_(input: I) -> ParseResult<Node, I> {
        sep_by1(Query::<I>::atom(), many1::<Vec<_>, _>(space())).map(|nodes| Node::Concat(nodes)).parse_lazy(input).into()
    }

    fn union() -> QueryParser<Node, I> {
        fn_parser(Query::<I>::union_, "union")
    }
    fn union_(input: I) -> ParseResult<Node, I> {
        chainl1(
            optional(Query::<I>::concat()).map(|opt| {
                match opt {
                    Some(opt) => opt,
                    None => Node::Empty,
                }
            }),
            token('|').map(|_| |left, right| Node::Union(Box::new(left), Box::new(right)))).parse_lazy(input).into()
    }

    fn group() -> QueryParser<Node, I> {
        fn_parser(Query::<I>::group_, "group")
    }
    fn group_(input: I) -> ParseResult<Node, I> {
        between(char('('), char(')'), Query::<I>::union()).map(|union| union).parse_lazy(input).into()
    }

    fn bare() -> QueryParser<Node, I> {
        fn_parser(Query::<I>::bare_, "bare")
    }
    fn bare_(input: I) -> ParseResult<Node, I> {
        try(Query::<I>::pattern()).or(Query::<I>::group()).parse_lazy(input).into()
    }

    fn atom() -> QueryParser<Node, I> {
        fn_parser(Query::<I>::atom_, "atom")
    }
    fn atom_(input: I) -> ParseResult<Node, I> {
        try(Query::<I>::star()).or(Query::<I>::bare()).parse_lazy(input).into()
    }

    fn star() -> QueryParser<Node, I> {
        fn_parser(Query::<I>::star_, "star")
    }
    fn star_(input: I) -> ParseResult<Node, I> {
        (Query::<I>::bare(), char('*')).map(|(atom, _)| Node::Star(Box::new(atom))).parse_lazy(input).into()
    }

    fn value() -> FnPtrParser<Node, I> {
        parser(Query::<I>::value_ as fn(_) -> _)
    }
    fn value_(input: I) -> ParseResult<Node, I> {
        Query::<I>::union().skip(eof()).parse_lazy(input).into()
    }
}

fn optimize(node: Node) -> Node {
    match node {
        Node::Pattern(_) => node,
        Node::Star(child) => Node::Star(Box::new(optimize(*child))),
        Node::Concat(nodes) => {
            if nodes.len() == 1 {
                optimize(nodes.into_iter().next().unwrap())
            } else {
                Node::Concat(nodes.into_iter().map(optimize).collect())
            }
        },
        Node::Union(left, right) => Node::Union(Box::new(optimize(*left)), Box::new(optimize(*right))),
        Node::Empty => node,
    }
}

pub fn parse(query_str: &str) -> Node {
    let mut parser = Query::value();
    let (node, _) = parser.parse(query_str).unwrap();
    optimize(node)
}

type ISeq = LinkedList<InstCode>;

pub fn compile(node: Node) -> Vec<InstCode> {
    fn asm(node: Node, pc: usize) -> (ISeq, usize) {
        match node {
            Node::Pattern(feat_ids) => {
                let mut inst_codes: ISeq = feat_ids.into_iter().enumerate().filter_map(|(i, v)| {
                    if let Some(id) = v {
                        Some((i, id))
                    } else {
                        None
                    }
                }).map(|(col, id)| InstCode::Expect(col, id)).collect();
                inst_codes.push_back(InstCode::Next);
                let len = inst_codes.len();
                (inst_codes, pc + len)
            },
            Node::Concat(nodes) => {
                nodes.into_iter().fold((ISeq::new(), pc), |(mut iseq, pc): (ISeq, usize), node| {
                    let (mut iseq2, pc2) = asm(node, pc);
                    iseq.append(&mut iseq2);
                    (iseq, pc2)
                })
            }
            Node::Union(left, right) => {
                let (mut a_iseq, a_pc) = asm(*left, pc + 1);
                let (mut b_iseq, b_pc) = asm(*right, a_pc + 1);
                let mut iseq = ISeq::new();
                iseq.push_back(InstCode::Split(pc + 1, a_pc + 1));
                iseq.append(&mut a_iseq);
                iseq.push_back(InstCode::Jump(b_pc));
                iseq.append(&mut b_iseq);
                (iseq, b_pc)
            },
            Node::Star(child) => {
                let (mut o_iseq, o_pc) = asm(*child, pc + 1);
                let mut iseq = ISeq::new();
                iseq.push_back(InstCode::Split(pc + 1, o_pc + 1));
                iseq.append(&mut o_iseq);
                iseq.push_back(InstCode::Jump(pc));
                (iseq, o_pc + 1)
            },
            Node::Empty => (ISeq::new(), pc),
        }
    }

    let (mut iseq, _) = asm(node, 0);
    iseq.push_back(InstCode::Match);
    iseq.into_iter().collect()
}