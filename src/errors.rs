// module erreurs -- j'ai tout mis ici c'est plus simple
// ParseError -> ce que le joueur tape de travers
// GameError  -> erreurs logiques pendant la partie

#[derive(Debug)]
pub enum ParseError {
    UnknownCommand(String),
    MissingArgument,
    InvalidNumber,  // genre "choose abc" au lieu de "choose 2"
}

#[derive(Debug)]
pub enum GameError {
    InvalidChoice(usize),
    MissingItem(String),
    GameOver, // hp <= 0
}

// Display pour avoir des messages lisibles en sortie utilisateur
// requis par les criteres qualite du TP
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnknownCommand(c) => write!(f, "commande inconnue: '{}'", c),
            ParseError::MissingArgument  => write!(f, "argument manquant"),
            ParseError::InvalidNumber    => write!(f, "numero invalide, entrez un chiffre"),
        }
    }
}

impl std::fmt::Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameError::InvalidChoice(n)  => write!(f, "le choix {} n'existe pas", n),
            GameError::MissingItem(obj)  => write!(f, "objet manquant: {}", obj),
            GameError::GameOver          => write!(f, "GAME OVER - plus de PV"),
        }
    }
}

// j'ai essaye d'implementer std::error::Error pour pouvoir utiliser l'operateur ?
// mais ca posait des problemes avec les trait objects et les lifetimes
// Display suffit pour ce TP
// impl std::error::Error for GameError {}
// impl std::error::Error for ParseError {}

// utilisee pendant le developpement pour afficher les erreurs, a nettoyer
pub fn fmt_parse_err(e: &ParseError) -> String {
    format!("[erreur commande] {}", e)
}
