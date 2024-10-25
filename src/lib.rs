use std::{
    collections::{HashMap, HashSet},
    ops::Index,
};

pub type Id = usize;
pub type Multiset = HashMap<Id, usize>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

// #[derive(Debug, Clone)]
// pub enum Math {
//     Const(i64),
//     Var(String),
//     Op(Op, [Id; 2]),
// }

#[derive(Debug, Clone)]
pub enum MathAC {
    Const(i64),
    Var(String),
    Op(Op, [Id; 2]),
    OpAC(Op, Multiset),
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Const(i64),
    Var(String),
    Multiset(String),
    Op(Op, Vec<Pattern>),
}

pub struct Arena(Vec<MathAC>);

impl Arena {
    pub fn new() -> Self {
        Arena(vec![])
    }

    pub fn insert(&mut self, expr: MathAC) -> Id {
        let id = self.0.len();
        self.0.push(expr);
        id
    }

    pub fn display(&self) {
        for (i, e) in self.0.iter().enumerate() {
            println!("{i}: {e:?}");
        }
    }
}

impl Index<Id> for Arena {
    type Output = MathAC;
    fn index(&self, index: Id) -> &Self::Output {
        &self.0[index]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubstValue {
    Atom(Id),
    Multiset(Multiset),
}

pub type Subst = HashMap<String, SubstValue>;

pub fn ac_match(expr: Id, pattern: &Pattern, arena: &Arena, subst: &mut Subst) -> bool {
    match (pattern, &arena[expr]) {
        (Pattern::Const(a), MathAC::Const(b)) if a == b => true,
        (Pattern::Var(x), _) => {
            if let Some(bound) = subst.get(x) {
                *bound == SubstValue::Atom(expr)
            } else {
                subst.insert(x.clone(), SubstValue::Atom(expr));
                true
            }
        }
        (Pattern::Op(pop, pats), MathAC::Op(eop, args)) if pop == eop => {
            assert!(pats.len() == 2);
            ac_match(args[0], &pats[0], arena, subst) && ac_match(args[1], &pats[1], arena, subst)
        }
        (Pattern::Op(pop, pats), MathAC::OpAC(eop, multiset)) if pop == eop => {
            // TODO: sort patterns by generality
            let mut remaining: Multiset = multiset.clone();
            for pat in pats {
                match pat {
                    Pattern::Multiset(xs) => {
                        // Bind the rest
                        assert!(!subst.contains_key(xs));
                        subst.insert(xs.clone(), SubstValue::Multiset(remaining.clone()));
                        return true;
                    }
                    _ => {
                        let mut matched: Option<Id> = None;
                        for &e in remaining.keys() {
                            // Don't like this clone
                            let mut try_subst = subst.clone();
                            if ac_match(e, pat, arena, &mut try_subst) {
                                *subst = try_subst;
                                matched = Some(e);
                                break;
                            }
                        }
                        if let Some(e) = matched {
                            remaining.remove(&e);
                        } else {
                            return false;
                        }
                    }
                }
            }
            true
        }
        (Pattern::Multiset(_), _) => panic!("top level multiset pattern not allowed!"),
        _ => false,
    }
}

fn multiset(vs: Vec<Id>) -> Multiset {
    let mut res: Multiset = Multiset::new();
    for v in vs {
        *res.entry(v).or_insert(0) += 1;
    }
    res
}

fn display_subst(subst: &Subst, arena: &Arena) {
    for (k, v) in subst {
        print!("{k} |-> ");
        match v {
            SubstValue::Atom(i) => println!("{:?}", &arena[*i]),
            SubstValue::Multiset(multiset) => {
                print!("{{");
                for (&i, &count) in multiset {
                    for _ in 0..count {
                        print!(" {:?}", &arena[i]);
                    }
                }
                println!(" }}");
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pattern() {
        let zero = Pattern::Const(0);
        let x = Pattern::Var("x".to_string());
        let xs = Pattern::Multiset("xs".to_string());
        let add_zero = Pattern::Op(Op::Add, vec![zero.clone(), x, xs.clone()]);
        let mul_zero = Pattern::Op(Op::Mul, vec![zero, xs]);

        println!("{add_zero:?}");
        println!("{mul_zero:?}");
    }

    #[test]
    fn test_match() {
        let zero = Pattern::Const(0);
        let x = Pattern::Var("x".to_string());
        let xs = Pattern::Multiset("xs".to_string());
        let add_zero = Pattern::Op(Op::Add, vec![zero.clone(), x, xs.clone()]);
        let mul_zero = Pattern::Op(Op::Mul, vec![zero, xs]);

        let mut arena = Arena::new();
        let ex = arena.insert(MathAC::Var("x".to_string()));
        let ey = arena.insert(MathAC::Var("y".to_string()));
        let e0 = arena.insert(MathAC::Const(0));
        let e1 = arena.insert(MathAC::Const(1));

        let expr = arena.insert(MathAC::OpAC(Op::Add, multiset(vec![ex, e1, e0, ey])));
        let expr2 = arena.insert(MathAC::OpAC(Op::Mul, multiset(vec![ex, e1, e0, ey])));

        {
            let mut subst: Subst = Subst::new();
            assert!(ac_match(expr, &add_zero, &arena, &mut subst));
            arena.display();
            display_subst(&subst, &arena);
        }

        {
            let mut subst: Subst = Subst::new();
            assert!(ac_match(expr2, &mul_zero, &arena, &mut subst));
            arena.display();
            display_subst(&subst, &arena);
        }
    }
}
