mod regex;
mod regex_nfa;

fn main() {
    let regexn = regex_nfa::regex_to_rNFA("a?a?a?a?a?a?a?a?a?a?a?a?a?a?a?a?a?a?a?a?a?a?a?a?a?a?a?a?a?a?aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
    let result = regex_nfa::match_regex_vec(regexn, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    println!("match? {}", result);
}
