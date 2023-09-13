use std::collections::HashMap;

use log::{debug, warn};
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
            MessageHandler,
        },
        sendprop::SendPropIdentifier,
        vector::VectorXY,
    },
    MessageType, ParserState,
};

pub type AnalyzerResult = (
    Vec<BombAttempt>,
    HashMap<UserId, tf_demo_parser::demo::parser::analyser::UserInfo>,
);

#[derive(Default, Debug)]
pub struct BombAttemptAnalyzer {
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
pub struct BombAttempt {
    pub user: u16,
    pub start_tick: DemoTick,
    pub land_tick: Option<DemoTick>,
    pub damage: u16,
    pub damage_taken: u16,
    pub state: BombState,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum BombState {
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

impl MessageHandler for BombAttemptAnalyzer {
    type Output = AnalyzerResult;

    fn does_handle(message_type: tf_demo_parser::MessageType) -> bool {
        matches!(
            message_type,
            MessageType::GameEvent | MessageType::NetTick | MessageType::PacketEntities
        )
    }

    fn into_output(self, _state: &ParserState) -> Self::Output {
        (self.jumps, self.users)
    }

    fn handle_message(&mut self, message: &Message, tick: DemoTick, state: &ParserState) {
        match message {
            Message::PacketEntities(message) => {
                for entity in &message.entities {
                    if entity.in_pvs {
                        self.handle_entity(entity, state);
                    }
                }
            }
            Message::GameEvent(message) => self.handle_game_event(&message.event, tick),
            Message::NetTick(_) => {
                self.jumpers.retain(|_, attempt| {
                    let cond = match attempt.land_tick {
                        Some(land_tick) => u32::from(tick) - u32::from(land_tick) < 66 * 2,
                        None => true,
                    };

                    if !cond {
                        attempt.state = BombState::TimedOut;
                        debug!("Timing out {:?}", attempt);
                        self.jumps.push(attempt.clone());
                    }

                    cond
                });
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

impl BombAttemptAnalyzer {
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
                debug!("Tick: {:?}, Start: {:?}", tick, e);
            }
            GameEvent::TeamPlayRoundWin(_) => self.round_state.is_dead_time = true,
            GameEvent::TeamPlayRoundStalemate(_) => self.round_state.is_dead_time = true,
            GameEvent::TeamPlayPointCaptured(e) => {
                debug!("Tick: {:?}, Capture: {:?}", tick, e);
            }
            GameEvent::PlayerChargeDeployed(e) => {
                debug!("Tick: {:?}, Charge: {:?}", tick, e);
            }
            GameEvent::RocketJumpLanded(e) => {
                if !self.should_record() {
                    return;
                }

                if self.jumpers.contains_key(&e.user_id) {
                    debug!("Tick: {:?} ", tick);
                    if !self.in_range(e.user_id) {
                        debug!(
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
                    debug!("Saving {:?}", attempt);
                    self.jumps.push(attempt);
                }
            }
            GameEvent::PlayerHurt(e) => {
                if !self.should_record() {
                    return;
                }

                self.jumpers.entry(e.attacker).and_modify(|x| {
                    if e.user_id != e.attacker {
                        x.damage += e.damage_amount;
                    }
                });

                self.jumpers.entry(e.user_id).and_modify(|x| {
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
        // FIXME: understand why this would ever be none, getting weird error logs on 2 demos
        let player: &Player = match self.player_data.get(&id.into()) {
            Some(p) => p,
            None => {
                warn!("THIS SHOULDN'T EVER HAPPEN! Failed to get player {}", id);
                return false;
            }
        };
        let pos = player.position;
        let mut min = f32::MAX;
        self.player_data.iter().for_each(|(_, p)| {
            if player.team == p.team {
                return;
            }
            let dist =
                f32::sqrt(f32::powi(p.position.x - pos.x, 2) + f32::powi(p.position.y - pos.y, 2));
            min = f32::min(min, dist);
        });
        debug!(
            "Closest player to {} is {} far away",
            self.users.get(&id.into()).unwrap().name,
            min
        );
        min < 500f32
    }
}
