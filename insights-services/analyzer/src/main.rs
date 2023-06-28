use std::fs;

use tf_demo_parser::{
    demo::{
        gamevent::GameEvent,
        message::Message,
        parser::{gamestateanalyser::UserId, handler::BorrowMessageHandler, MessageHandler},
    },
    Demo, DemoParser, MatchState, MessageType, ParserState,
};

// What I am trying to do right now: Sac Efficiency Calculator
// What is a sac? When soldier bombs into the other team (usually for the medic or demo)
// When does this happen? On even uber situations/stalemates (x amount of time since last cap, most players are alive), Disadvantaged situations

// Ways to identify stalemates:
// I guess first identify stalemates so write the analyzer
// Base case: soldier dies when all other 11 players are alive on similar uber scenarios

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::read("process_froyo.dem")?;
    let demo = Demo::new(&file);
    // let parser = DemoParser::new(demo.get_stream());

    let parser = DemoParser::new_with_analyser(demo.get_stream(), StalemateAnalyzer::new());
    let (_header, _state) = parser.parse()?;

    println!("{:?}", _state.users);
    // println!("Header: {:#?}\n\n", header);
    // println!("Rounds: {:#?}", state);
    Ok(())
}

#[derive(Default, Debug)]
struct Stalemate {
    length: u32,
    end_tick: u32,
}

#[derive(Default, Debug)]
struct StalemateAnalyzer {
    state: MatchState,
    stalemates: Vec<Stalemate>,
    ticks_since_event: u32,
}

impl BorrowMessageHandler for StalemateAnalyzer {
    fn borrow_output(&self, _state: &ParserState) -> &Self::Output {
        &self.state
    }
}

impl MessageHandler for StalemateAnalyzer {
    type Output = MatchState;

    fn does_handle(message_type: tf_demo_parser::MessageType) -> bool {
        matches!(message_type, MessageType::GameEvent | MessageType::NetTick)
    }

    fn into_output(self, _state: &ParserState) -> Self::Output {
        self.state
    }

    fn handle_message(&mut self, message: &Message, tick: u32) {
        match message {
            Message::GameEvent(message) => self.handle_game_event(&message.event, tick),
            Message::NetTick(_) => self.ticks_since_event += 1,
            _ => {}
        }
    }
}

impl StalemateAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    fn handle_game_event(&mut self, event: &GameEvent, tick: u32) {
        match event {
            GameEvent::TeamPlayRoundStart(e) => {
                println!("Tick: {}, Start: {:?}", tick, e);
                self.ticks_since_event = 0
            }
            GameEvent::TeamPlayPointCaptured(e) => {
                println!("Tick: {}, Capture: {:?}", tick, e);
                self.ticks_since_event = 0
            }
            GameEvent::PlayerChargeDeployed(e) => {
                println!("Tick: {}, Charge: {:?}", tick, e);
                self.ticks_since_event = 0
            }
            GameEvent::RocketJumpLanded(e) => {
                // println!("Tick: {}, Jump: {:?}", tick, e);
            }
            GameEvent::PlayerDeath(e) => {}
            _ => {}
        }
    }
}
