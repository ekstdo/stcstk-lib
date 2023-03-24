
pub enum  AST {
    Letter(char),
    Optional(Box<AST>),
    ZeroOrMore(Box<AST>),
    OneOrMore(Box<AST>),
    Alternate(Box<AST>, Box<AST>),
    Concat(Box<AST>, Box<AST>),
}

impl std::str::FromStr for AST {

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Pass::parse(s)
    }

    type Err = ParseError;
}

type ParseError = (String, usize);

pub struct Pass<'a> {
    s: std::iter::Peekable<std::str::CharIndices<'a>>,
}

impl <'a> Pass<'a> {
    pub fn parse(c: &'a str) -> Result<AST, ParseError>{
        let mut p = Pass { s: c.char_indices().peekable() };
        p.parse_regex()
    }

    fn expect(&mut self, c: char) -> Result<(), ParseError> {
        match self.s.next() {
            Some((_, a)) if a == c => Ok(()),
            Some((i, _)) => Err((format!("expected {}", c), i)),
            None => Err((format!("expected {}", c), 0))
        }
    }

    fn next_if(&mut self, c: char) -> bool {
        self.s.next_if(|&(_, a): &(usize, char)| a == c).is_some()
    }

    fn within(&mut self, c: &str) -> bool {
        c.contains(self.s.peek().map(|x| x.1).unwrap_or(c.chars().next().unwrap()))
    }

    fn parse_regex(&mut self) -> Result<AST, ParseError> {
        let mut lhs = self.parse_concat()?;
        while self.next_if('|') {
            let rhs = self.parse_concat()?;
            lhs = AST::Alternate(Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_concat(&mut self) -> Result<AST, ParseError> {
        let mut lhs = self.parse_symbol()?;
        while !self.within("|)") {
            let rhs = self.parse_symbol()?;
            lhs = AST::Concat(Box::new(lhs), Box::new(rhs))
        }

        Ok(lhs)
    }

    fn parse_symbol(&mut self) -> Result<AST, ParseError> {
        let mut lhs = self.parse_atom()?;
        while let Some((_, c)) = self.s.next_if(|&(_, a): &(usize, char)| "?+*".contains(a)) {
            match c {
                '+' => lhs = AST::OneOrMore(Box::new(lhs)),
                '*' => lhs = AST::ZeroOrMore(Box::new(lhs)),
                '?' => lhs = AST::Optional(Box::new(lhs)),
                _ => ()
            }
        }
        Ok(lhs)
    }

    fn parse_atom(&mut self) -> Result<AST, ParseError> {
        match self.s.next().ok_or(("Missing atom".to_string(), 0))? {
            (i, '(') => {
                let el = self.parse_regex()?;
                self.expect(')').map_err(|_: _| ("Unmatched (".to_string(), i))?;
                Ok(el)
            },
            (_, '\\') => Ok(AST::Letter(self.s.next().ok_or(("Missing atom".to_string(), 0))?.1)),
            (_, c) => Ok(AST::Letter(c))
        }
    }
}

