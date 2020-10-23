use jsona::emitter::Emitter;
use jsona::lexer::{Lexer, Position};
use jsona::loader::Loader;
use jsona::parser::{Event, EventReceiver, Parser};

const INPUT: &str = include_str!("spec/test_jsona_example.jsona");

#[test]
fn test_lex() {
    let expect = include_str!("spec/test_jsona_example_tok.txt");
    let mut target = String::new();
    let lexer = Lexer::new(INPUT.chars());
    for tok in lexer.into_iter() {
        target.push_str(&format!("{:?}\n", tok))
    }
    assert_eq!(expect, target)
}

struct EventCollector {
    evs: Vec<(Event, Position)>,
}

impl EventCollector {
    fn new() -> Self {
        Self { evs: Vec::new() }
    }
    #[allow(dead_code)]
    fn evs(&self) -> &Vec<(Event, Position)> {
        &self.evs
    }
}

impl EventReceiver for EventCollector {
    fn on_event(&mut self, event: Event, position: Position) {
        self.evs.push((event, position))
    }
}

#[test]
fn test_parse() {
    let expect = include_str!("spec/test_jsona_example_event.txt");
    let mut ec = EventCollector::new();
    let mut parser = Parser::new(INPUT.chars());
    parser.parse(&mut ec).unwrap();
    let mut target = String::new();
    for (ev, pos) in ec.evs {
        target.push_str(&format!("({:?}, {:?})\n", ev, pos))
    }
    assert_eq!(expect, target)
}

#[test]
fn test_emit() {
    let expect = include_str!("spec/test_jsona_example_emit.jsona");

    let result = Loader::load_from_str(INPUT).unwrap();
    let mut target = String::new();
    {
        let mut emitter = Emitter::new(&mut target);
        emitter.emit(&result).unwrap();
    }
    assert_eq!(expect, target)
}
