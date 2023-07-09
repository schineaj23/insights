use std::{collections::HashMap, fs};

use tf_demo_parser::{
    demo::{
        data::{DemoTick, UserInfo},
        gamevent::GameEvent,
        message::{
            packetentities::{EntityId, PacketEntity},
            Message,
        },
        parser::{gamestateanalyser::UserId, handler::BorrowMessageHandler, MessageHandler},
        sendprop::SendPropIdentifier,
    },
    Demo, DemoParser, MessageType, ParserState,
};

// What I am trying to do right now: Sac Efficiency Calculator
// What is a sac? When soldier bombs into the other team (usually for the medic or demo)
// When does this happen? On even uber situations/stalemates (x amount of time since last cap, most players are alive)
// , Disadvantaged situations

// Ways to identify stalemates:
// I guess first identify stalemates so write the analyzer
// Base case: soldier dies when all other 11 players are alive on similar uber scenarios

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::read("process_froyo.dem")?;
    let demo = Demo::new(&file);
    // let parser = DemoParser::new(demo.get_stream());

    let analyzer = InsightsAnalyzer::new();
    let parser = DemoParser::new_with_analyser(demo.get_stream(), analyzer);
    let (_header, bomb_attempts) = parser.parse()?;

    let bomb_dmg: Vec<&BombAttempt> = bomb_attempts
        .iter()
        .filter(|attempt| attempt.damage > 0)
        .collect();
    println!("Stalemates: {:?}", bomb_attempts);
    println!("Num dmg>0: {}", bomb_dmg.len());
    println!("Num Total: {}", bomb_attempts.len());

    // println!("Header: {:#?}\n\n", header);
    // println!("Rounds: {:#?}", state);
    Ok(())
}

#[allow(dead_code)]
#[derive(Debug)]
struct Stalemate {
    length: u32,
    start_tick: DemoTick,
    end_tick: DemoTick,
    end_event: GameEvent,
}

// Bomb efficiency
// Starts on RocketJumpLanded.
// If kill/opposing medic force/ then successful within threshold
// If die without any of these conditions (within threshold), unsucessful.

// things to consider: jumps that were just for reposition/spamming
// how do i make sure that they have jumped enough into the teams?

// Jump in
// Deal damage
// (potentially force/get kill/etc)
// die (or live?)

#[allow(dead_code)]
#[derive(Default, Debug)]
struct InsightsAnalyzer {
    users: HashMap<UserId, tf_demo_parser::demo::parser::analyser::UserInfo>,
    stalemates: Vec<Stalemate>,
    ticks_since_event: u32,
    ticks_since_death: u32,

    jumps: Vec<BombAttempt>,
    jumpers: HashMap<u16, BombAttempt>,

    class_names: Vec<tf_demo_parser::demo::packet::datatable::ServerClassName>,
    player_data: HashMap<UserId, Player>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
struct BombAttempt {
    user: u16,
    start_tick: DemoTick,
    land_tick: DemoTick,
    damage: u16,
    state: BombState,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum BombState {
    InProgress,
    TimedOut,
    _NewJumpStarted,
    Died,
}

#[derive(Default, Debug)]
struct Player {
    on_ground: bool,
}

impl BorrowMessageHandler for InsightsAnalyzer {
    fn borrow_output(&self, _state: &ParserState) -> &Self::Output {
        &self.jumps
    }
}

impl MessageHandler for InsightsAnalyzer {
    type Output = Vec<BombAttempt>;

    fn does_handle(message_type: tf_demo_parser::MessageType) -> bool {
        matches!(
            message_type,
            MessageType::GameEvent | MessageType::NetTick | MessageType::PacketEntities
        )
    }

    fn into_output(self, _state: &ParserState) -> Self::Output {
        self.jumps
    }
    fn handle_message(&mut self, message: &Message, tick: DemoTick, state: &ParserState) {
        match message {
            Message::PacketEntities(message) => {
                for entity in &message.entities {
                    self.handle_entity(entity, state);
                }
            }
            Message::GameEvent(message) => self.handle_game_event(&message.event, tick),
            Message::NetTick(_) => {
                self.ticks_since_event += 1;
                self.ticks_since_death += 1;

                self.jumpers.retain(|_, attempt| {
                    let p = self.player_data.get(&attempt.user.into()).unwrap();
                    if u32::from(tick) - u32::from(attempt.land_tick) < 66 * 5 || !p.on_ground {
                        true
                    } else {
                        attempt.state = BombState::TimedOut;
                        println!("Removing attempt {:?}", attempt);
                        self.jumps.push(attempt.clone());
                        false
                    }
                });
            }
            Message::ClassInfo(e) => {
                println!("{:?}", e)
            }
            _ => {}
        }
    }

    // Taken from demostf parser analyzer example
    // Trying to understand all these data structures lmao
    fn handle_string_entry(
        &mut self,
        table: &str,
        index: usize,
        entry: &tf_demo_parser::demo::packet::stringtable::StringTableEntry,
        _parser_state: &ParserState,
    ) {
        if table != "userinfo" {
            return;
        }

        if let Some(info) = UserInfo::parse_from_string_table(
            index as u16,
            entry.text.as_ref().map(|s| s.as_ref()),
            entry.extra_data.as_ref().map(|data| data.data.clone()),
        )
        .unwrap()
        {
            self.users
                .entry(info.player_info.user_id)
                .and_modify(|info| {
                    info.entity_id = info.entity_id;
                })
                .or_insert_with(|| info.into());
        }
    }

    fn handle_data_tables(
        &mut self,
        _tables: &[tf_demo_parser::demo::packet::datatable::ParseSendTable],
        server_classes: &[tf_demo_parser::demo::packet::datatable::ServerClass],
        _parser_state: &ParserState,
    ) {
        self.class_names = server_classes
            .iter()
            .map(|class| &class.name)
            .cloned()
            .collect();
    }
}

impl InsightsAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    fn _report_stalemate(&mut self, event: &GameEvent, tick: DemoTick) {
        let treshold: u32 = 66 * 15;
        if self.ticks_since_event < treshold && self.ticks_since_death < treshold {
            println!(
                "report_stalemate called but conds not met. last_death:  {} last_event: {}",
                self.ticks_since_death, self.ticks_since_event
            );
            return;
        }

        let len = u32::max(self.ticks_since_death, self.ticks_since_event);
        let start = tick - len;

        println!(
            "Stalemate finished. Length: {:?}, Event end trigger: {:?}",
            len, event
        );

        self.stalemates.push(Stalemate {
            length: len,
            start_tick: start,
            end_tick: tick,
            end_event: event.clone(),
        });

        self.ticks_since_event = 0;
    }

    fn handle_game_event(&mut self, event: &GameEvent, tick: DemoTick) {
        match event {
            GameEvent::TeamPlayRoundStart(e) => {
                println!("Tick: {:?}, Start: {:?}", tick, e);
                self.ticks_since_event = 0;
                self.ticks_since_death = 0;
            }
            GameEvent::TeamPlayPointCaptured(e) => {
                println!("Tick: {:?}, Capture: {:?}", tick, e);
                // self.report_stalemate(event, tick);
            }
            GameEvent::PlayerChargeDeployed(e) => {
                println!("Tick: {:?}, Charge: {:?}", tick, e);
                // self.report_stalemate(event, tick);
            }
            GameEvent::RocketJumpLanded(e) => {
                self.jumpers.entry(e.user_id).and_modify(|attempt| {
                    let user = self.users.get(&e.user_id.into()).unwrap();
                    println!(
                        "Rocket Jump Landed: {}, TickSinceStart: {}",
                        u32::from(tick) - u32::from(attempt.start_tick),
                        user.name
                    );
                    attempt.land_tick = tick;
                });
            }
            GameEvent::RocketJump(e) => {
                let user = match self.users.get(&e.user_id.into()) {
                    Some(user) => user,
                    None => return,
                };
                println!("Rocket Jump Started for {}", user.name);

                if self.jumpers.contains_key(&e.user_id) {
                    // let mut attempt = self.jumpers.remove(&e.user_id).unwrap();
                    // attempt.state = BombState::NewJumpStarted;
                    // self.jumps.push(attempt);
                } else {
                    self.jumpers.insert(
                        e.user_id,
                        BombAttempt {
                            user: e.user_id,
                            start_tick: tick,
                            land_tick: DemoTick::from(0),
                            damage: 0,
                            state: BombState::InProgress,
                        },
                    );
                }
            }
            GameEvent::PlayerDeath(e) => {
                self.ticks_since_death = 0;

                if self.jumpers.contains_key(&e.user_id) {
                    let mut attempt = self.jumpers.remove(&e.user_id).unwrap();
                    attempt.state = BombState::Died;
                    self.jumps.push(attempt);
                    return;
                }

                if self.jumpers.contains_key(&e.attacker) {
                    let attacker = match self.users.get(&e.attacker.into()) {
                        Some(user) => user,
                        None => return,
                    };
                    let victim = match self.users.get(&e.user_id.into()) {
                        Some(user) => user,
                        None => return,
                    };
                    println!("Jumper {} killed user {}", attacker.name, victim.name);
                }
            }
            GameEvent::PlayerHurt(e) => {
                if !self.jumpers.contains_key(&e.attacker) || self.jumpers.contains_key(&e.user_id)
                {
                    return;
                }

                let attacker = match self.users.get(&e.attacker.into()) {
                    Some(user) => user,
                    None => return,
                };
                let victim = match self.users.get(&e.user_id.into()) {
                    Some(user) => user,
                    None => return,
                };
                self.jumpers.get_mut(&e.attacker).unwrap().damage += e.damage_amount;
                println!(
                    "Jumper {} damaged {} for {}",
                    attacker.name, victim.name, e.damage_amount
                );
            }
            // e => println!("{:?}", e),
            _ => {}
        }
    }

    // FIXME: use another data structure
    fn get_or_insert_player_from_entity_id(&mut self, entity_id: EntityId) -> &mut Player {
        let id = self
            .users
            .iter()
            .find(|(_, info)| info.entity_id == entity_id)
            .map(|(id, _)| id)
            .cloned()
            .unwrap_or_default();
        self.player_data
            .entry(id)
            .or_insert_with(|| Player::default())
    }

    fn handle_entity(&mut self, entity: &PacketEntity, parser_state: &ParserState) {
        let class_name = self
            .class_names
            .get(usize::from(entity.server_class))
            .map(|server_name| server_name.as_str())
            .unwrap_or("");
        match class_name {
            "CTFPlayer" => self.handle_player_entity(entity, parser_state),
            _ => {}
        }
    }

    fn handle_player_entity(&mut self, entity: &PacketEntity, parser_state: &ParserState) {
        let player = self.get_or_insert_player_from_entity_id(entity.entity_index);
        const FLAGS_PROP: SendPropIdentifier = SendPropIdentifier::new("DT_BasePlayer", "m_fFlags");
        const FL_ONGROUND: i64 = 1 << 0;

        for prop in entity.props(parser_state) {
            match prop.identifier {
                FLAGS_PROP => {
                    let flags = i64::try_from(&prop.value).unwrap_or_default();
                    player.on_ground = (flags | FL_ONGROUND) == 0;
                }
                _ => {}
            }
        }
    }
}
