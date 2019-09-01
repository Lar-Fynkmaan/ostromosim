use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum DamageCard {
    FacilityDamage(RoleName),
    FacilityDestruction(RoleName),
    InfrastructureDamage,
}

type EventCardID = String;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
enum BonusCard {
    Cancel(EventCardID, Vec<RoleName>),
    Build(RoleName),
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
enum EventCard {
    Problem(
        EventCardID,
        Vec<(RoleName, RoleName)>,
        Option<RoleName>, // override for damage
        Option<DamageCard>,
    ),
    NoProblem,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct Role {
    facilities: usize,
    facilites_damaged: usize,
    name: RoleName,
    resources: usize,
    acted: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum RoleName {
    Hab,
    Joul,
    Man,
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct GameState {
    bonus_cards_in_play: Vec<BonusCard>,
    event_deck: Vec<EventCard>,
    event_cards_in_play: Vec<EventCard>,
    bonus_deck: Vec<BonusCard>,
    damage_deck: Vec<DamageCard>,
    year_number: usize,
    roles: HashMap<RoleName, Role>,
    infra_damage: usize,
}

impl GameState {

    fn reinit_damage(&mut self) {
        let mut damage_deck = Vec::new();
        damage_deck.push(DamageCard::InfrastructureDamage);
        damage_deck.push(DamageCard::InfrastructureDamage);
        damage_deck.push(DamageCard::InfrastructureDamage);
        damage_deck.push(DamageCard::FacilityDamage(RoleName::Hab));
        damage_deck.push(DamageCard::FacilityDamage(RoleName::Man));
        damage_deck.push(DamageCard::FacilityDamage(RoleName::Joul));
        damage_deck.push(DamageCard::FacilityDestruction(RoleName::Hab));
        damage_deck.push(DamageCard::FacilityDestruction(RoleName::Man));
        damage_deck.push(DamageCard::FacilityDestruction(RoleName::Joul));
        damage_deck.shuffle(&mut thread_rng());
        self.damage_deck = damage_deck;
    }
    fn deal_bonus_card(&mut self) {
        self.bonus_deck
            .pop()
            .map(|card| self.bonus_cards_in_play.push(card));
    }

    fn resources_left_to_build(&self) -> bool {
        let mut resources_left = true;
        for (_, role) in &self.roles {
            println!("{} {:?}", role.resources, role.name);
            resources_left = resources_left && (role.resources > 0);
        }
        resources_left
    }

    fn deal_event_and_damage_cards(&mut self, num: usize) {
        for x in 0..num {
            println!("Dealing card {} of {:?} ", x, num);
            self.event_deck.pop().map(|card| match card {
                EventCard::Problem(cardid, roles, damage_override, _) => {
                    if self.damage_deck.len() == 0 {
                        self.reinit_damage();
                    }
                    let damage = self.damage_deck.pop();
                    damage.map(|damage_card| {
                        println!("{}, {:?} ", cardid, damage_card);
                        self.event_cards_in_play.push(EventCard::Problem(
                            cardid,
                            roles,
                            damage_override,
                            Some(damage_card),
                        ))
                    });
                }
                EventCard::NoProblem => {}
            });
        }
    }

    fn get_num_event_cards(&self) -> usize {
        return (self.year_number / 3) + 2;
    }
    fn get_unacted_roles(&self) -> Vec<(RoleName, usize)> {
        let mut roles = Vec::new();
        for (_role_name, role) in &self.roles {
            if !role.acted {
                roles.push((role.name, role.facilities))
            }
        }
        roles.shuffle(&mut thread_rng());
        roles
    }

    fn build(&mut self, role_to_build: RoleName, with_card: bool) {
        for (role_name, role) in self.roles.iter_mut() {
            //Remove resources if a card is not being used
            if !with_card {
                role.resources = role.resources - 1;
            }
            if role_name == &role_to_build {
                if role.facilities < 4 {
                    role.facilities = role.facilities + 1;
                }
                //Not strictly true but worth pointing out
                role.acted = true;
            }
        }
    }

    fn do_damage_card(&mut self, override_role: Option<RoleName>, damage_card: DamageCard) {
        match damage_card {
            DamageCard::InfrastructureDamage => self.infra_damage = self.infra_damage + 1,
            DamageCard::FacilityDamage(card_role) => {
                let damage_type = match override_role {
                    Some(override_rolename) => override_rolename,
                    None => card_role,
                };
                let opt_role = self.roles.get_mut(&damage_type);
                opt_role.map(|role| {
                    role.facilites_damaged = role.facilites_damaged + 1;
                });
            }
            DamageCard::FacilityDestruction(card_role) => {
                let damage_type = match override_role {
                    Some(override_rolename) => override_rolename,
                    None => card_role,
                };
                let opt_role = self.roles.get_mut(&damage_type);
                opt_role.map(|role| {
                    if role.facilities > 1 {
                        role.facilities = role.facilities - 1;
                    } else if role.facilities == 1 {
                        role.facilites_damaged = role.facilites_damaged + 1;
                    }
                });
            }
        }
    }

    fn deal_with_event(&mut self, event_card: EventCard) {
        //spend all resources for event
        match event_card {
            EventCard::Problem(_role, role_costs, _, _) => {
                for (role, cost) in role_costs {
                    self.roles.get_mut(&cost).map(|the_role_cost| {
                        the_role_cost.resources = the_role_cost.resources - 1;
                    });
                    self.roles.get_mut(&role).map(|the_role_acted| {
                        the_role_acted.acted = true;
                    });
                }
            }
            _ => {}
        }

        //set roleName acted
    }

    fn can_deal_with_event(&mut self, event_card: &EventCard) -> bool {
        let mut can_deal = true;
        match event_card {
            EventCard::Problem(_, role_costs, _, _) => {
                for (_, cost) in role_costs {
                    self.roles
                        .get(&cost)
                        .map(|the_role| can_deal = can_deal && (the_role.resources > 0));
                }
            }
            _ => {}
        }
        can_deal
    }
    fn spend_cancel_card(&mut self, role: RoleName, pos: usize) {
        //remove bonus card
        self.bonus_cards_in_play.remove(pos);
        self.roles.get_mut(&role).map(|role| {
            role.acted = true;
        });

        //set roleName acted
    }
    fn find_roles_with_cancel(&self, event_id: &EventCardID) -> (Vec<RoleName>, usize) {
        let mut roles = Vec::new();
        let mut pos = 0;
        let mut found_pos = 0;
        for bonus_card in &self.bonus_cards_in_play {
            match bonus_card {
                BonusCard::Cancel(id, cancel_roles) => {
                    if id == event_id {
                        roles = cancel_roles.clone();
                        println!("Found card {}", id)
                    }
                    found_pos = pos
               }
                _ => {}
            };
            pos = pos + 1;
        }
        (roles, found_pos)
    }
    fn find_role_with_build(&self, role: RoleName) -> Option<usize> {
        let mut pos = 0;
        let mut found_pos = 0;
        let mut found = false;
        for bonus_card in &self.bonus_cards_in_play {
            match bonus_card {
                BonusCard::Build(build_role) => {
                    if role == *build_role {
                        println!("Found card {:?}", build_role);
                        found = true;
                        found_pos = pos;
                    }
                }
                _ => {}
            };
            pos = pos + 1;
        }
        if found {
            Some(found_pos)
        } else {
            None
        }
    }
    fn build_using_bonus(&mut self, role: RoleName) {
        match self.find_role_with_build(role) {
            Some(loc) => {
                println!("Found building bonus card at {}", loc);
                self.build(role, true);
                self.bonus_cards_in_play.remove(loc);
            }
            _ => {}
        }
    }
    fn play_year(&mut self) {
        println!("Starting Good Phase");
        // Good Stuff
        self.deal_bonus_card();
        for (_name, role) in self.roles.iter_mut() {
            if role.facilities > role.facilites_damaged {
                role.resources = role.facilities - role.facilites_damaged;
            } else {
                role.resources = 0;
            }
            role.facilites_damaged = 0;
            role.acted = false;
        }

        println!("Starting Event Phase");

        // Event + Planning
        let num_event_cards_to_play = self.get_num_event_cards();
        
        println!("NumEventCards = {}", num_event_cards_to_play);
        self.deal_event_and_damage_cards(num_event_cards_to_play);

        // Action
        //Deal with Events
        println!("Starting Action Phase");

        // Deal with event cards
        // For event cards / check whether any bonus cards exist to remove
        let cards_in_play = self.event_cards_in_play.clone();
        for card in cards_in_play {
            let card_clone = card.clone();
            match card {
                EventCard::Problem(id, _roles_effected, damage_override, opt_damage) => {
                    match opt_damage {
                        Some(damage) => {
                            let (mut capable_roles, pos) = self.find_roles_with_cancel(&id);
                            if capable_roles.len() > 0 {
                                capable_roles.pop().map(|role| {
                                    println!("Playing card to deal with {:?}", card_clone);
                                    self.spend_cancel_card(role, pos);
                                });
                            } else if self.can_deal_with_event(&card_clone) {
                                println!("Spending resources to deal with {:?}", card_clone);
                                self.deal_with_event(card_clone);
                            } else {
                                println!("Failed to deal with {:?}", card_clone);
                                self.do_damage_card(damage_override, damage);
                            }
                        }
                        None => {}
                    }
                }
                EventCard::NoProblem => {}
            }
        }
        self.event_cards_in_play = Vec::new();

        while self.resources_left_to_build() {
            //Build something if you can
            let mut roles_left_to_play = self.get_unacted_roles();
            if roles_left_to_play.len() == 0 {
                break;
            }
            let first_role = roles_left_to_play.pop();
            first_role.map(|(role, _)| {
                self.build(role, false);
            });

            println!("Building!")
        }
        let roles_left_to_play  = self.get_unacted_roles();
        for (role, _) in roles_left_to_play {
            self.build_using_bonus(role);
        }

        // checkBonusCardForBuild
        self.year_number = self.year_number + 1
        // Cleanup
        //Not needed as good stuff over writes.
        //make sure no events in play
    }

    fn new() -> GameState {
        let mut event_deck = Vec::new();

        //Spacecold issue, hab cost
        event_deck.push(EventCard::Problem(
            "Spacecold".to_string(),
            vec![(RoleName::Hab, RoleName::Hab)],
            Some(RoleName::Hab),
            None,
        ));
        //Raiding issue, hab cost
        event_deck.push(EventCard::Problem(
            "Spacecold".to_string(),
            vec![(RoleName::Hab, RoleName::Man)],
            Some(RoleName::Hab),
            None,
        ));
        //Raiding issue, man cost
        event_deck.push(EventCard::Problem(
            "Spacecold".to_string(),
            vec![(RoleName::Hab, RoleName::Joul)],
            Some(RoleName::Hab),
            None,
        ));

        //Mutiny issue, hab cost
        event_deck.push(EventCard::Problem(
            "Mutiny".to_string(),
            vec![(RoleName::Hab, RoleName::Hab)],
            None,
            None,
        ));
        //Nanofab issue, hab cost
        event_deck.push(EventCard::Problem(
            "Mutiny".to_string(),
            vec![(RoleName::Hab, RoleName::Hab)],
            None,
            None,
        ));
        //Nanofab issue, man cost
        event_deck.push(EventCard::Problem(
            "Mutiny".to_string(),
            vec![(RoleName::Hab, RoleName::Hab)],
            None,
            None,
        ));

        //Nanofab issue, energy cost
        event_deck.push(EventCard::Problem(
            "Nanobug".to_string(),
            vec![(RoleName::Man, RoleName::Joul)],
            None,
            None,
        ));
        //Nanofab issue, hab cost
        event_deck.push(EventCard::Problem(
            "Nanobug".to_string(),
            vec![(RoleName::Man, RoleName::Hab)],
            None,
            None,
        ));
        //Nanofab issue, man cost
        event_deck.push(EventCard::Problem(
            "Nanobug".to_string(),
            vec![(RoleName::Man, RoleName::Man)],
            None,
            None,
        ));
        //Hab issue, energy cost
        event_deck.push(EventCard::Problem(
            "Raiding".to_string(),
            vec![(RoleName::Hab, RoleName::Joul)],
            None,
            None,
        ));
        //Hab issue, hab cost
        event_deck.push(EventCard::Problem(
            "Raiding".to_string(),
            vec![(RoleName::Hab, RoleName::Hab)],
            None,
            None,
        ));
        //Hab issue, man cost
        event_deck.push(EventCard::Problem(
            "Raiding".to_string(),
            vec![(RoleName::Hab, RoleName::Man)],
            None,
            None,
        ));

        //Joul issue, energy cost, Joul Override
        event_deck.push(EventCard::Problem(
            "Surge".to_string(),
            vec![(RoleName::Joul, RoleName::Joul)],
            Some(RoleName::Joul),
            None,
        ));
        //Joul issue, hab cost, Joul
        event_deck.push(EventCard::Problem(
            "Surge".to_string(),
            vec![(RoleName::Joul, RoleName::Hab)],
            Some(RoleName::Joul),
            None,
        ));
        //Nanofab issue, man cost, Joul override
        event_deck.push(EventCard::Problem(
            "Surge".to_string(),
            vec![(RoleName::Joul, RoleName::Man)],
            Some(RoleName::Joul),
            None,
        ));
        //Joul/Man issue, man/hab cost - meteor
        event_deck.push(EventCard::Problem(
            "Meteor".to_string(),
            vec![
                (RoleName::Joul, RoleName::Man),
                (RoleName::Man, RoleName::Hab),
            ],
            None,
            None,
        ));
        //Joul/Man issue, man/hab cost - meteor
        event_deck.push(EventCard::Problem(
            "Meteor".to_string(),
            vec![
                (RoleName::Joul, RoleName::Hab),
                (RoleName::Man, RoleName::Joul),
            ],
            None,
            None,
        ));
        //Joul/Man issue, man/hab cost - meteor
        event_deck.push(EventCard::Problem(
            "Meteor".to_string(),
            vec![
                (RoleName::Joul, RoleName::Joul),
                (RoleName::Man, RoleName::Man),
            ],
            None,
            None,
        ));
        //Joul/Man issue, man/hab cost - quake
        event_deck.push(EventCard::Problem(
            "Quake".to_string(),
            vec![
                (RoleName::Joul, RoleName::Man),
                (RoleName::Man, RoleName::Hab),
            ],
            None,
            None,
        ));
        //Joul/Man issue, man/hab cost - quake
        event_deck.push(EventCard::Problem(
            "Quake".to_string(),
            vec![
                (RoleName::Joul, RoleName::Hab),
                (RoleName::Man, RoleName::Joul),
            ],
            None,
            None,
        ));
        //Joul/Man issue, man/hab cost - quake
        event_deck.push(EventCard::Problem(
            "Quake".to_string(),
            vec![
                (RoleName::Joul, RoleName::Joul),
                (RoleName::Man, RoleName::Man),
            ],
            None,
            None,
        ));
        //Joul/Man/Hab issue, Joul/man/hab cost - system
        event_deck.push(EventCard::Problem(
            "Systemic".to_string(),
            vec![
                (RoleName::Joul, RoleName::Joul),
                (RoleName::Man, RoleName::Man),
                (RoleName::Hab, RoleName::Hab),
            ],
            None,
            None,
        ));
        //Joul/Man/Hab issue, man/hab/joul cost - system
        event_deck.push(EventCard::Problem(
            "Systemic".to_string(),
            vec![
                (RoleName::Joul, RoleName::Man),
                (RoleName::Man, RoleName::Hab),
                (RoleName::Hab, RoleName::Joul),
            ],
            None,
            None,
        ));
        //Joul/Man/Hab issue, Joul/man/hab cost - system
        event_deck.push(EventCard::Problem(
            "Systemic".to_string(),
            vec![
                (RoleName::Joul, RoleName::Hab),
                (RoleName::Man, RoleName::Joul),
                (RoleName::Hab, RoleName::Man),
            ],
            None,
            None,
        ));
        event_deck.push(EventCard::NoProblem);
        event_deck.push(EventCard::NoProblem);
        event_deck.push(EventCard::NoProblem);
        event_deck.shuffle(&mut thread_rng());

        let mut bonus_deck = Vec::new();
        bonus_deck.push(BonusCard::Build(RoleName::Man));
        bonus_deck.push(BonusCard::Build(RoleName::Man));
        bonus_deck.push(BonusCard::Build(RoleName::Hab));
        bonus_deck.push(BonusCard::Build(RoleName::Hab));
        bonus_deck.push(BonusCard::Build(RoleName::Joul));
        bonus_deck.push(BonusCard::Build(RoleName::Joul));
        bonus_deck.push(BonusCard::Cancel(
            "Systemic".to_string(),
            vec![RoleName::Hab, RoleName::Joul, RoleName::Man],
        ));
        bonus_deck.push(BonusCard::Cancel(
            "Systemic".to_string(),
            vec![RoleName::Hab, RoleName::Joul, RoleName::Man],
        ));
        bonus_deck.push(BonusCard::Cancel(
            "Quake".to_string(),
            vec![RoleName::Joul, RoleName::Man],
        ));
        bonus_deck.push(BonusCard::Cancel(
            "Quake".to_string(),
            vec![RoleName::Joul, RoleName::Man],
        ));
        bonus_deck.push(BonusCard::Cancel(
            "Meteor".to_string(),
            vec![RoleName::Joul, RoleName::Man],
        ));
        bonus_deck.push(BonusCard::Cancel(
            "Meteor".to_string(),
            vec![RoleName::Joul, RoleName::Man],
        ));
        bonus_deck.push(BonusCard::Cancel("Mutiny".to_string(), vec![RoleName::Hab]));
        bonus_deck.push(BonusCard::Cancel("Mutiny".to_string(), vec![RoleName::Hab]));
        bonus_deck.push(BonusCard::Cancel(
            "Raiding".to_string(),
            vec![RoleName::Hab],
        ));
        bonus_deck.push(BonusCard::Cancel(
            "Raiding".to_string(),
            vec![RoleName::Hab],
        ));
        bonus_deck.push(BonusCard::Cancel(
            "Spacecold".to_string(),
            vec![RoleName::Hab],
        ));
        bonus_deck.push(BonusCard::Cancel(
            "Spacecold".to_string(),
            vec![RoleName::Hab],
        ));
        bonus_deck.shuffle(&mut thread_rng());

        let mut damage_deck = Vec::new();
        damage_deck.push(DamageCard::InfrastructureDamage);
        damage_deck.push(DamageCard::InfrastructureDamage);
        damage_deck.push(DamageCard::InfrastructureDamage);
        damage_deck.push(DamageCard::FacilityDamage(RoleName::Hab));
        damage_deck.push(DamageCard::FacilityDamage(RoleName::Man));
        damage_deck.push(DamageCard::FacilityDamage(RoleName::Joul));
        damage_deck.push(DamageCard::FacilityDestruction(RoleName::Hab));
        damage_deck.push(DamageCard::FacilityDestruction(RoleName::Man));
        damage_deck.push(DamageCard::FacilityDestruction(RoleName::Joul));
        damage_deck.shuffle(&mut thread_rng());

        let mut roles = HashMap::new();
        roles.insert(
            RoleName::Man,
            Role {
                name: RoleName::Man,
                facilities: 1,
                facilites_damaged: 0,
                acted: false,
                resources: 0,
            },
        );
        roles.insert(
            RoleName::Joul,
            Role {
                name: RoleName::Joul,
                facilities: 1,
                facilites_damaged: 0,
                acted: false,
                resources: 0,
            },
        );
        roles.insert(
            RoleName::Hab,
            Role {
                name: RoleName::Hab,
                facilities: 1,
                facilites_damaged: 0,
                acted: false,
                resources: 0,
            },
        );

        println!("Event deck length {}", event_deck.len());

        let gs = GameState {
            bonus_cards_in_play: Vec::new(),
            event_deck: event_deck,
            event_cards_in_play: Vec::new(),
            bonus_deck: bonus_deck,
            damage_deck: damage_deck,
            year_number: 0,
            roles: roles,
            infra_damage: 0,
        };
        gs
    }
}

fn main() {
    let mut gs = GameState::new();
    for _n in 0..9 {
        if gs.infra_damage > 2 {
            println!("Kerblooey!");
            break;
        }
        gs.play_year();
    }
    println!("Game state at end {:?}", gs);
}
