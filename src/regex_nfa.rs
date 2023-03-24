// Represent offsets
//
// `Seq(c, i)` offsets the current state by i, if it encounters c
// if it doesn't, it discards it 
//
// `Split(a, b)` adds a new state to the NFA, with 2 different offsets 
// `a` and `b`
//
// The NFA matches, whenever a state reaches its length.
#[derive(Debug, Clone, PartialEq)]
enum StateOffset {
    Seq(char, isize),
    Split(isize, isize),
}

// Represents States, similar to `StateOffset`
//
// `Seq(c, i)` switches to state i, if it encounters c
// if it doesn't, it discards it 
//
// `Split(a, b)` makes new states `a` and `b`
//
// The NFA matches, whenever a state reaches its length.
#[derive(Debug, Clone, PartialEq)]
pub enum State {
    Seq(char, usize),
    Split(usize, usize),
}

impl State {
    pub fn is_seq(&self) -> bool {
        match self {
            State::Seq(_, _) => true,
            _ => false
        }
    }
}

use std::{collections::LinkedList, str::FromStr};

use crate::regex::*;


fn replace_match_with(states: &mut LinkedList<StateOffset>, to: isize) {
    let match_pos = (states.len() as isize);
    for (index, state) in states.iter_mut().enumerate() {
        use StateOffset::*;
        let index = index as isize;

        match state {
            &mut Seq(c, i) if index + i == match_pos => *state = Seq(c, to - index),
            &mut Split(i, a) if index + i == match_pos => *state = Split(to - index, a),
            &mut Split(a, i) if index + i == match_pos => *state = Split(a, to - index),
            _ => ()
        }
    }
}

fn regex_ast_to_rNFA(ast: AST) -> LinkedList<StateOffset> {
    match ast {
        AST::Letter(c) => LinkedList::from([StateOffset::Seq(c, 1)]),
        AST::Optional(c) => {
            let mut m = regex_ast_to_rNFA(*c);
            m.push_front(StateOffset::Split(1, m.len() as isize + 1));
            m
        },
        AST::ZeroOrMore(c) => { // potentially O(n) :/
            let mut c = regex_ast_to_rNFA(*c);
            replace_match_with(&mut c, -1);
            c.push_front(StateOffset::Split(1, c.len() as isize + 1));
            c
        },
        AST::OneOrMore(c) =>  {
            let mut m = regex_ast_to_rNFA(*c);
            m.push_back(StateOffset::Split(1, -(m.len() as isize)));
            m
        },
        AST::Alternate(a, b) => { // same here
            let mut a = regex_ast_to_rNFA(*a);
            let mut b = regex_ast_to_rNFA(*b);
            let l = a.len();
            replace_match_with(&mut a, (l + b.len()) as isize);
            a.push_front(StateOffset::Split(1, a.len() as isize + 1));
            a.append(&mut b);
            a
        },
        AST::Concat(a, b) => {
            let mut a = regex_ast_to_rNFA(*a);
            let mut b = regex_ast_to_rNFA(*b);
            a.append(&mut b);
            a
        },
    }
}


// Converts a regex string to a minimal NFA representation
pub fn regex_to_rNFA(s: &str) -> Result<Vec<State>, (String, usize)> {
    Ok(
        regex_ast_to_rNFA(AST::from_str(s)?)
            .into_iter()
            .enumerate()
            .map(|(index, s): (usize, StateOffset)| match s {
                StateOffset::Seq(c, o) => State::Seq(c, (o + index as isize) as usize),
                StateOffset::Split(a, b) => State::Split((a + index as isize) as usize, (b + index as isize) as usize),
            })
            .collect::<Vec<State>>()
    )
}


fn addstate(base: &Vec<State>, from: usize, to: &mut Vec<bool>){
    to[from] = true;
    if from == base.len() {
        return;
    }
    if let State::Split(a, b)  = base[from] {
        if !to[a] {
            addstate(base, a, to);
        }
        if !to[b] {
            addstate(base, b, to);
        }
    }
}

fn set_false(c: &mut Vec<bool>){
    for i in c.iter_mut() {
        *i = false;
    }
}

pub fn match_regex_vec(rvec: Vec<State>, s: &str) -> bool {
    let matchstate = rvec.len();
    let mut vec1: Vec<usize> = Vec::with_capacity(rvec.len());
    let mut vec2: Vec<bool> = vec![false; rvec.len() + 1];
    let mut last_matched = false;
    addstate(&rvec, 0, &mut vec2);
    for (index, i) in vec2.iter().enumerate() {
        if *i {
            if index == matchstate {
                last_matched = true;
                continue;
            }
            if rvec[index].is_seq() {
                vec1.push(index);
            }
        }
    }
    set_false(&mut vec2);

    for i in s.chars() {
        last_matched = false;
        while let Some(s) = vec1.pop() {
            match rvec[s] {
                State::Seq(c, o) if c == i => addstate(&rvec, o, &mut vec2),
                _ => ()
            }
        }
        vec1.clear();
        for (index, i) in vec2.iter().enumerate() {
            if *i {
                if index == matchstate {
                    last_matched = true;
                    continue;
                }
                if rvec[index].is_seq() {
                    vec1.push(index);
                }
            }
        }
        set_false(&mut vec2)
    }
    last_matched
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_simple(){
        let a = AST::from_str("a").unwrap();
        
        assert_eq!(regex_ast_to_rNFA(a), LinkedList::from([StateOffset::Seq('a', 1)]))
    }

    #[test]
    fn test_replace(){
        let mut a = LinkedList::from([StateOffset::Seq('a', 1), StateOffset::Seq('b', 1)]);
        replace_match_with(&mut a, 5);
        assert_eq!(a, LinkedList::from([StateOffset::Seq('a', 1), StateOffset::Seq('b', 4)]))
    }

    #[test]
    fn test_concat(){
        let a = AST::from_str("ab").unwrap();
        
        assert_eq!(regex_ast_to_rNFA(a), LinkedList::from([StateOffset::Seq('a', 1), StateOffset::Seq('b', 1)]))
    }

    #[test]
    fn test_opt(){
        let a = AST::from_str("a?").unwrap();
        
        assert_eq!(regex_ast_to_rNFA(a), LinkedList::from([StateOffset::Split(1, 2), StateOffset::Seq('a', 1)]))
    }

    #[test]
    fn test_oneormore(){
        let a = AST::from_str("a+").unwrap();
        
        assert_eq!(regex_ast_to_rNFA(a), LinkedList::from([StateOffset::Seq('a', 1), StateOffset::Split(1, -1)]))
    }

    #[test]
    fn test_zeroormore(){
        let a = AST::from_str("a*").unwrap();
        
        assert_eq!(regex_ast_to_rNFA(a), LinkedList::from([StateOffset::Split(1, 2), StateOffset::Seq('a', -1)]))
    }

    #[test]
    fn test_alter(){
        let a = AST::from_str("a|b").unwrap();
        
        assert_eq!(regex_ast_to_rNFA(a), LinkedList::from([StateOffset::Split(1, 2), StateOffset::Seq('a', 2), StateOffset::Seq('b', 1)]))
    }

    #[test]
    fn test_alter2(){
        let a = AST::from_str("(ab)|(cd)").unwrap();
        
        assert_eq!(regex_ast_to_rNFA(a), LinkedList::from([StateOffset::Split(1, 3), StateOffset::Seq('a', 1),  StateOffset::Seq('b', 3), StateOffset::Seq('c', 1), StateOffset::Seq('d', 1)]))
    }

    #[test]
    fn test_alter_vec(){
        let a = regex_to_rNFA("(ab)|(cd)").unwrap();
        
        assert_eq!(a, vec![State::Split(1, 3), State::Seq('a', 2),  State::Seq('b', 5), State::Seq('c', 4), State::Seq('d', 5)])
    }

    #[test]
    fn test_simple_match(){
        let a = regex_to_rNFA("a").unwrap();
   
        assert!(match_regex_vec(a, "a"))
    }

    #[test]
    fn test_simple_not_match(){
        let a = regex_to_rNFA("a").unwrap();
   
        assert!(!match_regex_vec(a, "aa"))
    }
}
