//! Ajuda de teoria para quem está começando: escala, pentatônica e acordes do tom do pad,
//! com os nomes das notas escritos do jeito certo (Fá maior tem Bb, não A#).

const LETTERS: [char; 7] = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];
const LETTER_PC: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];
const SOLFEJO: [&str; 7] = ["Dó", "Ré", "Mi", "Fá", "Sol", "Lá", "Si"];

/// Como escrever a tônica de cada tom (o mais usado na prática): (letra 0..6, acidente).
const MAJOR_TONIC: [(usize, i32); 12] = [
    (0, 0),  // C
    (1, -1), // Db
    (1, 0),  // D
    (2, -1), // Eb
    (2, 0),  // E
    (3, 0),  // F
    (3, 1),  // F#
    (4, 0),  // G
    (5, -1), // Ab
    (5, 0),  // A
    (6, -1), // Bb
    (6, 0),  // B
];
const MINOR_TONIC: [(usize, i32); 12] = [
    (0, 0),  // Cm
    (0, 1),  // C#m
    (1, 0),  // Dm
    (2, -1), // Ebm
    (2, 0),  // Em
    (3, 0),  // Fm
    (3, 1),  // F#m
    (4, 0),  // Gm
    (4, 1),  // G#m
    (5, 0),  // Am
    (6, -1), // Bbm
    (6, 0),  // Bm
];

const MAJOR_STEPS: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];
const MINOR_STEPS: [i32; 7] = [0, 2, 3, 5, 7, 8, 10];

/// Uma nota escrita: nome (ex.: "Bb"), nome em português (ex.: "Sib") e altura 0..11.
#[derive(Clone, Debug, PartialEq)]
pub struct Note {
    pub name: String,
    pub solfejo: String,
    pub pc: u8,
}

fn accidental(a: i32) -> &'static str {
    match a {
        -2 => "bb",
        -1 => "b",
        1 => "#",
        2 => "##",
        _ => "",
    }
}

fn accidental_pt(a: i32) -> &'static str {
    match a {
        -2 => "bb",
        -1 => "b",
        1 => "#",
        2 => "##",
        _ => "",
    }
}

/// Escala de 7 notas do tom `key` (0 = C ... 11 = B), maior ou menor natural.
pub fn scale(key: usize, minor: bool) -> Vec<Note> {
    let key = key % 12;
    let (letter, _) = if minor {
        MINOR_TONIC[key]
    } else {
        MAJOR_TONIC[key]
    };
    let steps = if minor { MINOR_STEPS } else { MAJOR_STEPS };
    (0..7)
        .map(|i| {
            let l = (letter + i) % 7;
            let pc = (key as i32 + steps[i]).rem_euclid(12);
            let mut a = (pc - LETTER_PC[l]).rem_euclid(12);
            if a > 6 {
                a -= 12;
            }
            Note {
                name: format!("{}{}", LETTERS[l], accidental(a)),
                solfejo: format!("{}{}", SOLFEJO[l], accidental_pt(a)),
                pc: pc as u8,
            }
        })
        .collect()
}

/// Nome do tom, ex.: "C", "Bb", "F#m".
pub fn key_name(key: usize, minor: bool) -> String {
    let s = scale(key, minor);
    format!("{}{}", s[0].name, if minor { "m" } else { "" })
}

/// Pentatônica (5 notas): maior = graus 1 2 3 5 6; menor = 1 3 4 5 7.
pub fn pentatonic(key: usize, minor: bool) -> Vec<Note> {
    let s = scale(key, minor);
    let pick: [usize; 5] = if minor {
        [0, 2, 3, 4, 6]
    } else {
        [0, 1, 2, 4, 5]
    };
    pick.iter().map(|&i| s[i].clone()).collect()
}

/// Acordes do tom (campo harmônico), um por grau da escala.
pub fn chords(key: usize, minor: bool) -> Vec<String> {
    let s = scale(key, minor);
    // maior: I ii iii IV V vi vii°   |   menor natural: i ii° III iv v VI VII
    let kinds: [&str; 7] = if minor {
        ["m", "°", "", "m", "m", "", ""]
    } else {
        ["", "m", "m", "", "", "m", "°"]
    };
    s.iter()
        .zip(kinds)
        .map(|(n, k)| format!("{}{}", n.name, k))
        .collect()
}

/// Sequências de acordes mais usadas no louvor (graus da escala, 1 = acorde do tom).
const MAJOR_PROGS: [(&str, &[usize]); 7] = [
    ("a campeã", &[1, 5, 6, 4]),
    ("começando triste", &[6, 4, 1, 5]),
    ("adoração", &[4, 1, 5, 6]),
    ("clássica / hino", &[1, 4, 5, 1]),
    ("balada", &[1, 6, 4, 5]),
    ("sobe pro refrão", &[4, 5, 6]),
    ("gospel", &[2, 5, 1]),
];
const MINOR_PROGS: [(&str, &[usize]); 4] = [
    ("épica", &[1, 6, 3, 7]),
    ("subindo", &[1, 6, 7]),
    ("clássica menor", &[1, 4, 5, 1]),
    ("ministração", &[6, 7, 1]),
];

/// Uma sequência pronta no tom: (nome, números "1 – 5 – 6 – 4", acordes "G – D – Em – C").
pub struct Progression {
    pub name: &'static str,
    pub numbers: String,
    pub chords: String,
}

pub fn progressions(key: usize, minor: bool) -> Vec<Progression> {
    let ch = chords(key, minor);
    let list: &[(&str, &[usize])] = if minor { &MINOR_PROGS } else { &MAJOR_PROGS };
    list.iter()
        .map(|(name, degs)| Progression {
            name,
            numbers: degs
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join(" – "),
            chords: degs
                .iter()
                .map(|d| ch[d - 1].clone())
                .collect::<Vec<_>>()
                .join(" – "),
        })
        .collect()
}

/// Altura (0..11) está na escala?
pub fn in_scale(key: usize, minor: bool, pc: u8) -> bool {
    scale(key, minor).iter().any(|n| n.pc == pc % 12)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(v: &[Note]) -> String {
        v.iter()
            .map(|n| n.name.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn major_scales_are_spelled_right() {
        assert_eq!(names(&scale(0, false)), "C D E F G A B");
        assert_eq!(names(&scale(5, false)), "F G A Bb C D E");
        assert_eq!(names(&scale(2, false)), "D E F# G A B C#");
        assert_eq!(names(&scale(10, false)), "Bb C D Eb F G A"); // botão "A#" vira Bb
        assert_eq!(names(&scale(6, false)), "F# G# A# B C# D# E#");
        assert_eq!(names(&scale(1, false)), "Db Eb F Gb Ab Bb C");
    }

    #[test]
    fn minor_scales_and_names() {
        assert_eq!(names(&scale(9, true)), "A B C D E F G");
        assert_eq!(names(&scale(0, true)), "C D Eb F G Ab Bb");
        assert_eq!(key_name(1, true), "C#m");
        assert_eq!(key_name(3, true), "Ebm");
        assert_eq!(key_name(10, false), "Bb");
    }

    #[test]
    fn pentatonic_and_chords() {
        assert_eq!(names(&pentatonic(0, false)), "C D E G A");
        assert_eq!(names(&pentatonic(9, true)), "A C D E G");
        assert_eq!(chords(0, false).join(" "), "C Dm Em F G Am B°");
        assert_eq!(chords(7, false).join(" "), "G Am Bm C D Em F#°");
        assert_eq!(chords(9, true).join(" "), "Am B° C Dm Em F G");
        assert!(in_scale(0, false, 4) && !in_scale(0, false, 1));
        assert_eq!(scale(0, false)[3].solfejo, "Fá");
        let g = progressions(7, false);
        assert_eq!(g[0].numbers, "1 – 5 – 6 – 4");
        assert_eq!(g[0].chords, "G – D – Em – C");
        assert_eq!(progressions(9, true)[0].chords, "Am – F – C – G");
        assert_eq!(scale(10, false)[0].solfejo, "Sib");
    }
}
