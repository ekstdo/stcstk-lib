use std::collections::HashMap;

// a simple DFA specifically for Regex
//
// it's being constructed as it's trying to match a string
pub struct LazyRegexDFA {
    convert: HashMap<Vec<bool>, usize>,
    inner: Vec<Vec<usize>>,
    reconvert: Vec<Vec<bool>>
}

impl LazyRegexDFA {
    pub fn new(nfa_vec: &Vec<bool>) -> Self {
        Self {
            convert: HashMap::new(),
            inner: vec![vec![0; 256]],
            reconvert: vec![vec![false;nfa_vec.len()]]
        }
    }

    pub fn new_state(&mut self, i: Vec<bool>) {
        self.convert.insert(i.clone(), self.inner.len());
        self.inner.push(vec![0; 256]);
        self.reconvert.push(i);
    }
}


