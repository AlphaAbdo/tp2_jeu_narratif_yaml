// Command Pattern -- une struct par commande, toutes implementent GameCommand
// c'est ce que le cours appelle "encapsuler une action dans un objet"
// cf. enonce section 4

use crate::errors::{GameError, ParseError};
use crate::scenario::Scenario;
use crate::state::GameState;

// ce que retourne execute() quand tout se passe bien
#[derive(Debug, PartialEq)]
pub enum CommandOutcome {
    Continue,
    Quit,
    Victory,
    Escape,
    Defeat,
}

// le trait que toutes les commandes doivent implementer
pub trait GameCommand {
    fn execute(
        &self,
        scenario: &Scenario,
        state: &mut GameState,
    ) -> Result<CommandOutcome, GameError>;
}

// ---------- structs ----------

pub struct LookCommand;
pub struct InventoryCommand;
pub struct StatusCommand;
pub struct QuitCommand;

pub struct ChooseCommand {
    pub index: usize, // numero saisi par le joueur, commence a 1
}

// ---------- implementations ----------

impl GameCommand for LookCommand {
    fn execute(&self, scenario: &Scenario, state: &mut GameState) -> Result<CommandOutcome, GameError> {
        let scene = scenario.get_scene(&state.current_scene)
            .expect("scene courante introuvable -- ca ne devrait pas arriver");

        println!("\n== {} ==", scene.title);
        println!("{}", scene.text);

        match &scene.choices {
            None => {
                // scene sans choix = scene terminale (ending)
                println!("[pas de choix disponibles]");
            }
            Some(choices) => {
                println!();
                for (i, c) in choices.iter().enumerate() {
                    if let Some(req) = &c.required_item {
                        println!("  {}. {} [necessite: {}]", i + 1, c.label, req);
                    } else {
                        println!("  {}. {}", i + 1, c.label);
                    }
                }
            }
        }

        Ok(CommandOutcome::Continue)
    }
}

impl GameCommand for ChooseCommand {
    fn execute(&self, scenario: &Scenario, state: &mut GameState) -> Result<CommandOutcome, GameError> {
        let scene = scenario.get_scene(&state.current_scene)
            .expect("scene introuvable");

        // pas de choix sur une scene terminale
        let choices = scene.choices.as_ref()
            .ok_or(GameError::InvalidChoice(self.index))?;

        // verifier que le numero est dans la plage valide (1-based)
        if self.index == 0 || self.index > choices.len() {
            return Err(GameError::InvalidChoice(self.index));
        }

        let choix = &choices[self.index - 1];

        // verifier l'inventaire si requis
        if let Some(req) = &choix.required_item {
            if !state.has_item(req) {
                return Err(GameError::MissingItem(req.clone()));
            }
        }

        // aller dans la scene suivante
        let next = scenario.get_scene(&choix.next)
            .expect("destination introuvable -- le validate() aurait du attraper ca");

        // ramasser un objet si la scene en a un
        if let Some(obj) = &next.found_item {
            state.pick_up(obj);
            println!("  -> vous ramassez: {}", obj);
        }

        // appliquer les degats ou soins
        if let Some(delta) = next.hp_delta {
            state.hp += delta;
            if delta < 0 {
                println!("  -> vous perdez {} PV  (reste: {})", delta.abs(), state.hp);
            } else {
                println!("  -> vous recup {} PV  (total: {})", delta, state.hp);
            }
        }

        state.current_scene = next.id.clone();

        // mourir avant de verifier l'ending
        if state.hp <= 0 {
            println!("\n{}", next.text);
            return Err(GameError::GameOver);
        }

        // scene de fin ?
        if let Some(end) = &next.ending {
            println!("\n{}", next.text);
            return match end.as_str() {
                "victory" => Ok(CommandOutcome::Victory),
                "escape"  => Ok(CommandOutcome::Escape),
                "defeat"  => Ok(CommandOutcome::Defeat),
                autre => {
                    // ending inconnu dans le YAML -- on continue quand meme
                    eprintln!("[warn] ending inconnu: '{}'", autre);
                    Ok(CommandOutcome::Continue)
                }
            };
        }

        // scene normale -> afficher
        LookCommand.execute(scenario, state)?;
        Ok(CommandOutcome::Continue)
    }
}

impl GameCommand for InventoryCommand {
    fn execute(&self, _scenario: &Scenario, state: &mut GameState) -> Result<CommandOutcome, GameError> {
        if state.inventory.is_empty() {
            println!("inventaire vide");
        } else {
            println!("inventaire: {}", state.inventory.join(", "));
        }
        Ok(CommandOutcome::Continue)
    }
}

impl GameCommand for StatusCommand {
    fn execute(&self, _scenario: &Scenario, state: &mut GameState) -> Result<CommandOutcome, GameError> {
        println!("PV: {}   scene: {}", state.hp, state.current_scene);
        Ok(CommandOutcome::Continue)
    }
}

impl GameCommand for QuitCommand {
    fn execute(&self, _scenario: &Scenario, _state: &mut GameState) -> Result<CommandOutcome, GameError> {
        Ok(CommandOutcome::Quit)
    }
}

// j'avais essaye de faire un systeme de "commande annulable" avec un stack
// mais c'est vraiment pas necessaire ici et ca compliquait tout
//
// struct CommandHistory { stack: Vec<Box<dyn GameCommand>> }
// impl CommandHistory {
//     fn push(&mut self, cmd: Box<dyn GameCommand>) { self.stack.push(cmd); }
//     fn undo(&mut self) { self.stack.pop(); } // pas sur de comment annuler proprement
// }

// transforme une ligne de texte en commande executable
pub fn parse_command(line: &str) -> Result<Box<dyn GameCommand>, ParseError> {
    let txt = line.trim();
    let mut parts = txt.splitn(2, ' ');
    let cmd = parts.next().unwrap_or("").to_lowercase();
    let arg = parts.next();

    match cmd.as_str() {
        "look"              => Ok(Box::new(LookCommand)),
        "inventory" | "inv" => Ok(Box::new(InventoryCommand)),
        "status"            => Ok(Box::new(StatusCommand)),
        "quit" | "exit"     => Ok(Box::new(QuitCommand)),
        "choose" => {
            let s = arg.ok_or(ParseError::MissingArgument)?;
            let n = s.trim().parse::<usize>()
                .map_err(|_| ParseError::InvalidNumber)?;
            Ok(Box::new(ChooseCommand { index: n }))
        }
        autre => Err(ParseError::UnknownCommand(autre.to_string())),
    }
}

// --------- tests ---------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::{Choice, Scene, Scenario};

    // je recopie ce helper partout dans mes tests, c'est pas terrible
    // TODO: le mettre dans un module de test commun peut-etre
    fn build_scenario() -> Scenario {
        Scenario {
            start_scene: "debut".to_string(),
            initial_hp: 10,
            scenes: vec![
                Scene {
                    id: "debut".to_string(),
                    title: "Debut".to_string(),
                    text: "vous etes au debut".to_string(),
                    choices: Some(vec![
                        Choice {
                            label: "aller en avant".to_string(),
                            next: "fin".to_string(),
                            required_item: None,
                        },
                        Choice {
                            label: "aller avec badge".to_string(),
                            next: "fin".to_string(),
                            required_item: Some("badge".to_string()),
                        },
                    ]),
                    found_item: None,
                    hp_delta: None,
                    ending: None,
                },
                Scene {
                    id: "fin".to_string(),
                    title: "Fin".to_string(),
                    text: "vous gagnez".to_string(),
                    choices: None,
                    found_item: None,
                    hp_delta: None,
                    ending: Some("victory".to_string()),
                },
                Scene {
                    id: "piege".to_string(),
                    title: "Piege".to_string(),
                    text: "vous mourez".to_string(),
                    choices: None,
                    found_item: None,
                    hp_delta: Some(-999),
                    ending: None,
                },
            ],
        }
    }

    #[test]
    fn test_look_parse() {
        assert!(parse_command("look").is_ok());
        assert!(parse_command("  look  ").is_ok()); // avec espaces
    }

    #[test]
    fn test_choose_parse_ok() {
        // juste verifier que ca parse bien le numero
        assert!(parse_command("choose 1").is_ok());
        assert!(parse_command("choose 99").is_ok());
    }

    #[test]
    fn test_choose_sans_numero() {
        let err = parse_command("choose").err().unwrap();
        assert!(matches!(err, ParseError::MissingArgument));
    }

    #[test]
    fn test_choose_mauvais_numero() {
        let err = parse_command("choose abc").err().unwrap();
        assert!(matches!(err, ParseError::InvalidNumber));
    }

    #[test]
    fn test_commande_inconnue() {
        let err = parse_command("sauter").err().unwrap();
        assert!(matches!(err, ParseError::UnknownCommand(_)));
    }

    #[test]
    fn test_chemin_victoire() {
        let sc = build_scenario();
        let mut state = GameState::new("debut", 10);
        let res = ChooseCommand { index: 1 }.execute(&sc, &mut state).unwrap();
        assert_eq!(res, CommandOutcome::Victory);
    }

    #[test]
    fn test_choix_hors_borne() {
        let sc = build_scenario();
        let mut state = GameState::new("debut", 10);
        let err = ChooseCommand { index: 99 }.execute(&sc, &mut state).unwrap_err();
        assert!(matches!(err, GameError::InvalidChoice(99)));
    }

    #[test]
    fn test_objet_manquant() {
        let sc = build_scenario();
        let mut state = GameState::new("debut", 10);
        // choix 2 necessite "badge" -- pas dans l'inventaire
        let err = ChooseCommand { index: 2 }.execute(&sc, &mut state).unwrap_err();
        assert!(matches!(err, GameError::MissingItem(_)));
    }

    #[test]
    fn test_game_over_hp_vide() {
        // scenario custom: une scene qui retire tous les PV
        let sc = Scenario {
            start_scene: "a".to_string(),
            initial_hp: 5,
            scenes: vec![
                Scene {
                    id: "a".to_string(),
                    title: "A".to_string(),
                    text: "debut".to_string(),
                    choices: Some(vec![Choice {
                        label: "tomber".to_string(),
                        next: "b".to_string(),
                        required_item: None,
                    }]),
                    found_item: None,
                    hp_delta: None,
                    ending: None,
                },
                Scene {
                    id: "b".to_string(),
                    title: "B".to_string(),
                    text: "vous etes mort".to_string(),
                    choices: None,
                    found_item: None,
                    hp_delta: Some(-100), // largement assez
                    ending: None,
                },
            ],
        };
        let mut state = GameState::new("a", 5);
        let err = ChooseCommand { index: 1 }.execute(&sc, &mut state).unwrap_err();
        assert!(matches!(err, GameError::GameOver));
    }
}
