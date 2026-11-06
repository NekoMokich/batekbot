//! Слэш-команды бота. Тексты описаний дословно из Python-версии.

pub mod misc;
pub mod music;
pub mod video;

use crate::{Data, Error};

pub fn all() -> Vec<poise::Command<Data, Error>> {
    vec![
        music::play(),
        music::stop(),
        video::timur(),
        video::batek(),
        video::victorovich(),
        misc::grandmobile(),
        misc::bomjara(),
        misc::roflvideo(),
        misc::help_command(),
        misc::kupyansk(),
        misc::batekfetch(),
    ]
}

/// Контракт поверхности команд: имена и описания дословно как в Python.
/// Тест ловит случайную правку документации, которая меняет текст в Discord.
#[cfg(test)]
mod tests {
    use super::*;

    const EXPECTED: &[(&str, &str)] = &[
        ("play", "Я НАФАРМИЛ КРИПТЫ КАК СКУФ"),
        ("stop", "ВЫКЛЮЧИ ЕГО НАХУЙ"),
        ("timur", "ЗАЧЕМ Я ПОМЕНЯЛ ЕЕ НА ЭТО НА ЧЕРНОБЕЛОЕ"),
        ("batek", "белые розы"),
        ("victorovich", "В СТАРОМ ПАРКЕ ПАХНЕТ ХВОЙНОЙ ТИШИНОЙ"),
        ("grandmobile", "грэндмобайл"),
        ("bomjara", "АКАКФИЛЬМНАНОУТБУКЕВКЛЮЧИТЬ"),
        (
            "roflvideo",
            "не смешная хуйня про 320 пошути же даун ебучий",
        ),
        ("help", "слєш хелп"),
        ("kupyansk", "купянскнаш"),
        ("batekfetch", "ЭЭЭФЕТЧИ"),
    ];

    #[test]
    fn surface_matches_python() {
        let commands = all();
        assert_eq!(commands.len(), EXPECTED.len(), "число команд");

        for (name, description) in EXPECTED {
            let command = commands
                .iter()
                .find(|c| c.name == *name)
                .unwrap_or_else(|| panic!("нет команды {name}"));
            assert_eq!(
                command.description.as_deref(),
                Some(*description),
                "описание команды {name}"
            );
        }
    }
}
