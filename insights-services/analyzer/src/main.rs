use std::{collections::HashMap, fs, time::Instant};

use tf_demo_parser::{
    demo::{
        data::{DemoTick, UserInfo},
        gamevent::GameEvent,
        message::{
            packetentities::{EntityId, PacketEntity},
            Message,
        },
        parser::{
            gamestateanalyser::{Team, UserId},
            handler::BorrowMessageHandler,
            MessageHandler,
        },
        sendprop::SendPropIdentifier,
        vector::VectorXY,
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
    let file = fs::read("bagel_froyo.dem")?;
    let demo = Demo::new(&file);
    // let parser = DemoParser::new(demo.get_stream());

    let start = Instant::now();

    let analyzer = InsightsAnalyzer::new();
    let parser = DemoParser::new_with_analyser(demo.get_stream(), analyzer);
    let (_header, bomb_attempts) = parser.parse()?;

    let t = start.elapsed().as_secs_f32();

    println!("All attempts");
    bomb_attempts.iter().for_each(|x| {
        println!("{:?}", x);
    });

    let bomb_dmg: Vec<&BombAttempt> = bomb_attempts
        .iter()
        .filter(|attempt| attempt.damage > 0)
        .collect();

    let mut dmg: HashMap<u16, (i32, i32, u32)> = HashMap::new();

    let mut a = 0;
    bomb_attempts.iter().for_each(|x| {
        a += x.damage;
        dmg.entry(x.user)
            .and_modify(|u| {
                u.0 += 1;
                u.1 += i32::from(x.damage > 0);
                u.2 += x.damage as u32;
            })
            .or_insert((1, 0, x.damage as u32));
    });

    for (uid, (cnt, cnt_dmg, dmg)) in dmg {
        println!(
            "Uid: {}, NumBombs: {}, Dmg/Bomb: {}, NonzeroDmgBombs: {}",
            uid,
            cnt,
            dmg as f64 / cnt as f64,
            cnt_dmg
        );
    }

    println!(
        "Average dmg/bombattempt: {}",
        (a as f32 / bomb_attempts.len() as f32)
    );

    // println!("Stalemates: {:?}", bomb_attempts);
    println!("Num dmg>0: {}", bomb_dmg.len());
    println!("Num Total: {}", bomb_attempts.len());
    println!("Took {} seconds to analyze demo.", t);

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

    jumps: Vec<BombAttempt>,
    jumpers: HashMap<u16, BombAttempt>,

    class_names: Vec<tf_demo_parser::demo::packet::datatable::ServerClassName>,
    player_data: HashMap<UserId, Player>,

    round_state: RoundState,
}

#[derive(Default, Debug)]
struct RoundState {
    has_match_started: bool,
    is_dead_time: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
struct BombAttempt {
    user: u16,
    start_tick: DemoTick,
    land_tick: Option<DemoTick>,
    damage: u16,
    damage_taken: u16,
    state: BombState,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum BombState {
    InProgress,
    TimedOut,
    NewJumpStarted,
    Died,
}

#[derive(Default, Debug)]
struct Player {
    on_ground: bool,
    position: VectorXY,
    team: Team,
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
                self.jumpers.retain(|_, attempt| {
                    // let p = self.player_data.get(&attempt.user.into()).unwrap();
                    let cond = match attempt.land_tick {
                        Some(land_tick) => u32::from(tick) - u32::from(land_tick) < 66 * 3,
                        None => true,
                    };

                    if !cond {
                        attempt.state = BombState::TimedOut;
                        println!("Timing out {:?}", attempt);
                        self.jumps.push(attempt.clone());
                    }

                    cond
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
                .and_modify(|i| {
                    i.entity_id = info.entity_id;
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

    fn should_record(&self) -> bool {
        return self.round_state.has_match_started && !self.round_state.is_dead_time;
    }

    fn handle_game_event(&mut self, event: &GameEvent, tick: DemoTick) {
        match event {
            GameEvent::TeamPlayRoundStart(e) => {
                if !self.round_state.has_match_started {
                    self.round_state.has_match_started = true;
                }
                self.round_state.is_dead_time = false;
                println!("Tick: {:?}, Start: {:?}", tick, e);
            }
            GameEvent::TeamPlayRoundWin(_) => self.round_state.is_dead_time = true,
            GameEvent::TeamPlayRoundStalemate(_) => self.round_state.is_dead_time = true,
            GameEvent::TeamPlayPointCaptured(e) => {
                println!("Tick: {:?}, Capture: {:?}", tick, e);
            }
            GameEvent::PlayerChargeDeployed(e) => {
                println!("Tick: {:?}, Charge: {:?}", tick, e);
            }
            GameEvent::RocketJumpLanded(e) => {
                if !self.should_record() {
                    return;
                }

                // FIXME: remove this if it is bad
                if self.jumpers.contains_key(&e.user_id) {
                    print!("Tick: {:?} ", tick);
                    if !self.in_range(e.user_id) {
                        println!(
                            "Removing ({}) {} because not in range.",
                            e.user_id,
                            self.users.get(&e.user_id.into()).unwrap().name
                        );
                        self.jumpers.remove(&e.user_id);
                    }
                }

                self.jumpers.entry(e.user_id).and_modify(|attempt| {
                    attempt.land_tick = Some(tick);
                });
            }
            GameEvent::RocketJump(e) => {
                if !self.should_record() {
                    return;
                }

                if self.jumpers.contains_key(&e.user_id) {
                    self.jumpers.entry(e.user_id).and_modify(|attempt| {
                        attempt.state = BombState::NewJumpStarted;
                    });
                } else {
                    self.jumpers.insert(
                        e.user_id,
                        BombAttempt {
                            user: e.user_id,
                            start_tick: tick,
                            land_tick: None,
                            damage: 0,
                            damage_taken: 0,
                            state: BombState::InProgress,
                        },
                    );
                }
            }
            GameEvent::PlayerDeath(e) => {
                if !self.should_record() {
                    return;
                }

                if self.jumpers.contains_key(&e.user_id) {
                    let mut attempt = self.jumpers.remove(&e.user_id).unwrap();
                    attempt.state = BombState::Died;
                    println!("Saving {:?}", attempt);
                    self.jumps.push(attempt);
                    return;
                }
            }
            GameEvent::PlayerHurt(e) => {
                if !self.jumpers.contains_key(&e.attacker)
                    || !self.jumpers.contains_key(&e.user_id)
                    || !self.should_record()
                {
                    return;
                }

                self.jumpers.entry(e.attacker).and_modify(|x| {
                    if e.user_id != e.attacker {
                        x.damage += e.damage_amount;
                    }
                });

                self.jumpers.entry(e.user_id).and_modify(|x| {
                    if e.attacker != e.user_id {}
                    // TODO: FIXME: NOTE: this is including the damage taken from the blast itself
                    x.damage_taken += e.damage_amount;
                });
            }
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
        const TEAM_PROP: SendPropIdentifier =
            SendPropIdentifier::new("DT_BaseEntity", "m_iTeamNum");
        const LOCAL_ORIGIN: SendPropIdentifier =
            SendPropIdentifier::new("DT_TFLocalPlayerExclusive", "m_vecOrigin");
        const NON_LOCAL_ORIGIN: SendPropIdentifier =
            SendPropIdentifier::new("DT_TFNonLocalPlayerExclusive", "m_vecOrigin");
        const FL_ONGROUND: i64 = 1 << 0;

        for prop in entity.props(parser_state) {
            match prop.identifier {
                FLAGS_PROP => {
                    let flags = i64::try_from(&prop.value).unwrap_or_default();
                    player.on_ground = (flags | FL_ONGROUND) == 0;
                }
                LOCAL_ORIGIN | NON_LOCAL_ORIGIN => {
                    player.position = VectorXY::try_from(&prop.value).unwrap_or_default();
                }
                TEAM_PROP => {
                    player.team = Team::new(i64::try_from(&prop.value).unwrap_or_default())
                }
                _ => {}
            }
        }
    }

    // TODO: possibly separate waddling skips vs high bombs, since small skip/spam gets detected if < 750, but fades dont?

    // Gets the distance to the closest player by iterating over every enemy and taking pythagorean distance
    fn in_range(&self, id: u16) -> bool {
        let player: &Player = self.player_data.get(&id.into()).unwrap();
        let pos = player.position;
        let mut min = f32::MAX;
        self.player_data.iter().for_each(|(_, p)| {
            if player.team == p.team || p.team == Team::Other || p.team == Team::Spectator {
                return;
            }
            let dist =
                f32::sqrt(f32::powi(p.position.x - pos.x, 2) + f32::powi(p.position.y - pos.y, 2));
            min = f32::min(min, dist);
        });
        println!(
            "Closest player to {} is {} far away",
            self.users.get(&id.into()).unwrap().name,
            min
        );
        min < 500f32
    }
}
