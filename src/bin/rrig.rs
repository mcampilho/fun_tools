//! rrig: random identity generator (like rig) for test data: name, address,
//! city and phone, for the United States, Portugal or Spain. Everything is
//! made up: random combinations of common names, streets and cities.

use fun::cli::{Arg, Cli};
use fun::i18n::Lang;
use fun::rng::Rng;

struct Texts {
    help: &'static str,
    unknown_country: &'static str,
}

const EN: Texts = Texts {
    help: "\
Usage: rrig [options]

Makes up random identities (name, address, phone) for test data.
All of it is fictional: random mixes of common names, streets and cities.

  -c, --count <n>      How many identities (default 1)
  -C, --country <cc>   us (United States), pt (Portugal), es (Spain)
                       (default: from the language: en=us, pt=pt, es=es)
  -m, --male           Only male names
  -f, --female         Only female names
  -e, --email          Add an e-mail address (at example.com/.org/.net)
  -o, --format <f>     text, csv or json (default text)
  -s, --seed <n>       Seed, for the same identities again
      --lang <code>    Language: en, pt, es (also: RTOOLS_LANG)
  -h, --help           Show this help
  -V, --version        Show the version
",
    unknown_country: "unknown country",
};

const PT: Texts = Texts {
    help: "\
Utilização: rrig [opções]

Inventa identidades ao acaso (nome, morada, telefone) para dados de teste.
É tudo fictício: misturas ao acaso de nomes, ruas e cidades comuns.

  -c, --count <n>      Quantas identidades (por omissão 1)
  -C, --country <cc>   us (Estados Unidos), pt (Portugal), es (Espanha)
                       (por omissão: conforme o idioma: en=us, pt=pt, es=es)
  -m, --male           Só nomes masculinos
  -f, --female         Só nomes femininos
  -e, --email          Acrescentar um endereço de e-mail (em example.com/.org/.net)
  -o, --format <f>     text, csv ou json (por omissão text)
  -s, --seed <n>       Semente, para voltar a obter as mesmas identidades
      --lang <código>  Idioma: en, pt, es (também: RTOOLS_LANG)
  -h, --help           Mostrar esta ajuda
  -V, --version        Mostrar a versão
",
    unknown_country: "país desconhecido",
};

const ES: Texts = Texts {
    help: "\
Uso: rrig [opciones]

Inventa identidades al azar (nombre, dirección, teléfono) para datos de prueba.
Todo es ficticio: mezclas al azar de nombres, calles y ciudades comunes.

  -c, --count <n>      Cuántas identidades (por defecto 1)
  -C, --country <cc>   us (Estados Unidos), pt (Portugal), es (España)
                       (por defecto: según el idioma: en=us, pt=pt, es=es)
  -m, --male           Solo nombres masculinos
  -f, --female         Solo nombres femeninos
  -e, --email          Añadir una dirección de correo (en example.com/.org/.net)
  -o, --format <f>     text, csv o json (por defecto text)
  -s, --seed <n>       Semilla, para obtener las mismas identidades
      --lang <código>  Idioma: en, pt, es (también: RTOOLS_LANG)
  -h, --help           Mostrar esta ayuda
  -V, --version        Mostrar la versión
",
    unknown_country: "país desconocido",
};

struct Country {
    code: &'static str,
    male: &'static [&'static str],
    female: &'static [&'static str],
    surnames: &'static [&'static str],
    /// How many surnames a full name has.
    surname_count: usize,
    street_types: &'static [&'static str],
    street_names: &'static [&'static str],
    /// (city, region, postal code prefix, phone area code)
    cities: &'static [(&'static str, &'static str, &'static str, &'static str)],
}

const US: Country = Country {
    code: "us",
    male: &[
        "James", "Robert", "John", "Michael", "David", "William", "Richard", "Joseph", "Thomas", "Charles",
        "Daniel", "Matthew", "Anthony", "Mark", "Steven", "Paul", "Andrew", "Joshua", "Kevin", "Brian",
        "George", "Edward", "Ryan", "Jacob", "Gary", "Eric", "Nicholas", "Jonathan", "Peter", "Samuel",
    ],
    female: &[
        "Mary", "Patricia", "Jennifer", "Linda", "Elizabeth", "Barbara", "Susan", "Jessica", "Sarah", "Karen",
        "Lisa", "Nancy", "Betty", "Sandra", "Margaret", "Ashley", "Emily", "Donna", "Michelle", "Carol",
        "Amanda", "Melissa", "Deborah", "Rebecca", "Laura", "Helen", "Anna", "Grace", "Olivia", "Emma",
    ],
    surnames: &[
        "Smith", "Johnson", "Williams", "Brown", "Jones", "Miller", "Davis", "Wilson", "Anderson", "Taylor",
        "Thomas", "Moore", "Martin", "Jackson", "Thompson", "White", "Harris", "Clark", "Lewis", "Robinson",
        "Walker", "Young", "Allen", "King", "Wright", "Scott", "Hill", "Green", "Adams", "Baker",
        "Nelson", "Carter", "Mitchell", "Roberts", "Turner", "Phillips", "Campbell", "Parker", "Evans", "Collins",
    ],
    surname_count: 1,
    street_types: &["St", "Ave", "Rd", "Blvd", "Ln", "Dr", "Ct", "Way", "Pl"],
    street_names: &[
        "Main", "Oak", "Maple", "Cedar", "Elm", "Pine", "Washington", "Lake", "Hill", "Park", "Sunset",
        "Lincoln", "Jackson", "River", "Highland", "Forest", "Church", "Spring", "Meadow", "Willow", "Chestnut",
        "Franklin", "Walnut", "Center", "Mill", "Ridge", "Valley", "Jefferson", "Madison", "Prospect",
    ],
    cities: &[
        ("New York", "NY", "100", "212"), ("Los Angeles", "CA", "900", "213"), ("Chicago", "IL", "606", "312"),
        ("Houston", "TX", "770", "713"), ("Phoenix", "AZ", "850", "602"), ("Philadelphia", "PA", "191", "215"),
        ("San Antonio", "TX", "782", "210"), ("San Diego", "CA", "921", "619"), ("Dallas", "TX", "752", "214"),
        ("Austin", "TX", "787", "512"), ("Seattle", "WA", "981", "206"), ("Denver", "CO", "802", "303"),
        ("Boston", "MA", "021", "617"), ("Portland", "OR", "972", "503"), ("Atlanta", "GA", "303", "404"),
        ("Miami", "FL", "331", "305"), ("Springfield", "IL", "627", "217"), ("Madison", "WI", "537", "608"),
        ("Columbus", "OH", "432", "614"), ("Nashville", "TN", "372", "615"),
    ],
};

const PORTUGAL: Country = Country {
    code: "pt",
    male: &[
        "João", "José", "António", "Francisco", "Manuel", "Pedro", "Tiago", "Rui", "Miguel", "Luís",
        "Carlos", "Paulo", "Jorge", "Ricardo", "Nuno", "André", "Bruno", "Diogo", "Rodrigo", "Gonçalo",
        "Duarte", "Tomás", "Afonso", "Vasco", "Hugo", "Filipe", "Sérgio", "Fernando", "Martim", "Henrique",
    ],
    female: &[
        "Maria", "Ana", "Joana", "Inês", "Beatriz", "Mariana", "Catarina", "Sofia", "Rita", "Marta",
        "Sara", "Teresa", "Isabel", "Carla", "Patrícia", "Filipa", "Leonor", "Matilde", "Carolina", "Helena",
        "Raquel", "Margarida", "Luísa", "Cláudia", "Susana", "Diana", "Francisca", "Alice", "Lara", "Madalena",
    ],
    surnames: &[
        "Silva", "Santos", "Ferreira", "Pereira", "Oliveira", "Costa", "Rodrigues", "Martins", "Jesus", "Sousa",
        "Fernandes", "Gonçalves", "Gomes", "Lopes", "Marques", "Alves", "Almeida", "Ribeiro", "Pinto", "Carvalho",
        "Teixeira", "Moreira", "Correia", "Mendes", "Nunes", "Soares", "Vieira", "Monteiro", "Cardoso", "Rocha",
        "Neves", "Coelho", "Cruz", "Cunha", "Pires", "Ramos", "Reis", "Simões", "Antunes", "Matos",
    ],
    surname_count: 2,
    street_types: &["Rua", "Avenida", "Travessa", "Largo", "Praça", "Calçada", "Estrada"],
    street_names: &[
        "da Liberdade", "das Flores", "do Comércio", "da Paz", "de São João", "do Sol", "da Escola",
        "dos Pescadores", "da Estação", "do Mar", "das Amoreiras", "da República", "dos Combatentes",
        "de Camões", "da Igreja", "do Castelo", "das Oliveiras", "da Fonte", "do Rossio", "de Santo António",
        "da Boavista", "do Carmo", "das Laranjeiras", "do Vale", "da Ribeira", "dos Moinhos", "da Alegria",
    ],
    cities: &[
        ("Lisboa", "", "1", "21"), ("Porto", "", "4", "22"), ("Coimbra", "", "3", "239"), ("Braga", "", "47", "253"),
        ("Faro", "", "80", "289"), ("Aveiro", "", "38", "234"), ("Évora", "", "70", "266"), ("Setúbal", "", "29", "265"),
        ("Viseu", "", "35", "232"), ("Leiria", "", "24", "244"), ("Funchal", "", "90", "291"),
        ("Ponta Delgada", "", "95", "296"), ("Guimarães", "", "48", "253"), ("Bragança", "", "53", "273"),
        ("Castelo Branco", "", "60", "272"), ("Viana do Castelo", "", "49", "258"),
    ],
};

const SPAIN: Country = Country {
    code: "es",
    male: &[
        "Antonio", "Manuel", "José", "Francisco", "David", "Juan", "Javier", "Daniel", "Carlos", "Jesús",
        "Alejandro", "Miguel", "Rafael", "Pablo", "Sergio", "Fernando", "Jorge", "Luis", "Alberto", "Álvaro",
        "Diego", "Adrián", "Raúl", "Enrique", "Rubén", "Iván", "Andrés", "Óscar", "Hugo", "Mario",
    ],
    female: &[
        "María", "Carmen", "Ana", "Laura", "Isabel", "Lucía", "Cristina", "Marta", "Elena", "Sara",
        "Paula", "Raquel", "Pilar", "Rosa", "Beatriz", "Silvia", "Julia", "Nuria", "Irene", "Alba",
        "Patricia", "Andrea", "Mercedes", "Rocío", "Teresa", "Claudia", "Lorena", "Inés", "Sofía", "Noelia",
    ],
    surnames: &[
        "García", "Rodríguez", "González", "Fernández", "López", "Martínez", "Sánchez", "Pérez", "Gómez", "Martín",
        "Jiménez", "Ruiz", "Hernández", "Díaz", "Moreno", "Muñoz", "Álvarez", "Romero", "Alonso", "Gutiérrez",
        "Navarro", "Torres", "Domínguez", "Vázquez", "Ramos", "Gil", "Ramírez", "Serrano", "Blanco", "Molina",
        "Morales", "Suárez", "Ortega", "Delgado", "Castro", "Ortiz", "Rubio", "Marín", "Sanz", "Iglesias",
    ],
    surname_count: 2,
    street_types: &["Calle", "Avenida", "Paseo", "Plaza", "Camino", "Ronda", "Travesía"],
    street_names: &[
        "Mayor", "de la Paz", "del Sol", "de las Flores", "de Cervantes", "Real", "de la Constitución",
        "del Carmen", "de San Juan", "de la Iglesia", "del Mar", "de Goya", "de la Estación", "Nueva",
        "de los Olivos", "del Río", "de Andalucía", "de la Libertad", "del Prado", "de Castilla",
        "de la Fuente", "del Pilar", "de Colón", "de la Alameda", "de Velázquez", "del Parque",
    ],
    cities: &[
        ("Madrid", "", "28", "91"), ("Barcelona", "", "08", "93"), ("Valencia", "", "46", "96"),
        ("Sevilla", "", "41", "95"), ("Zaragoza", "", "50", "976"), ("Málaga", "", "29", "952"),
        ("Bilbao", "", "48", "94"), ("Valladolid", "", "47", "983"), ("Vigo", "", "36", "986"),
        ("Granada", "", "18", "958"), ("Salamanca", "", "37", "923"), ("Palma", "", "07", "971"),
        ("Alicante", "", "03", "965"), ("Córdoba", "", "14", "957"), ("Oviedo", "", "33", "985"),
        ("Santander", "", "39", "942"),
    ],
};

const COUNTRIES: &[&Country] = &[&US, &PORTUGAL, &SPAIN];

struct Person {
    name: String,
    street: String,
    city_line: String,
    phone: String,
    email: Option<String>,
}

fn digits(rng: &mut Rng, n: usize) -> String {
    (0..n).map(|_| char::from(b'0' + rng.below(10) as u8)).collect()
}

/// Lower-case ASCII version of a name, for e-mail addresses.
fn ascii(s: &str) -> String {
    s.chars()
        .filter_map(|c| {
            let c = match c {
                'á' | 'à' | 'â' | 'ã' | 'Á' | 'À' | 'Â' | 'Ã' => 'a',
                'é' | 'ê' | 'É' | 'Ê' => 'e',
                'í' | 'Í' => 'i',
                'ó' | 'ô' | 'õ' | 'Ó' | 'Ô' | 'Õ' => 'o',
                'ú' | 'ü' | 'Ú' | 'Ü' => 'u',
                'ç' | 'Ç' => 'c',
                'ñ' | 'Ñ' => 'n',
                c => c,
            };
            c.is_ascii_alphanumeric().then(|| c.to_ascii_lowercase())
        })
        .collect()
}

fn person(c: &Country, rng: &mut Rng, gender: Option<bool>, email: bool) -> Person {
    let male = gender.unwrap_or_else(|| rng.chance(0.5));
    let first = *rng.pick(if male { c.male } else { c.female });
    let mut surnames: Vec<&str> = Vec::new();
    while surnames.len() < c.surname_count {
        let s = *rng.pick(c.surnames);
        if !surnames.contains(&s) {
            surnames.push(s);
        }
    }
    let name = format!("{first} {}", surnames.join(" "));
    let (city, region, prefix, area) = *rng.pick(c.cities);
    let number = rng.range(1, if c.code == "us" { 9999 } else { 250 });
    let (street, city_line, phone) = match c.code {
        "us" => (
            format!("{number} {} {}", rng.pick(c.street_names), rng.pick(c.street_types)),
            format!("{city}, {region}  {prefix}{}", digits(rng, 2)),
            // 555-0100 to 555-0199 are reserved for fiction.
            format!("({area}) 555-01{}", digits(rng, 2)),
        ),
        "pt" => {
            let cp4 = format!("{prefix}{}", digits(rng, 4 - prefix.len()));
            let floor = if rng.chance(0.5) { format!(", {}º {}", rng.range(1, 9), if rng.chance(0.5) { "Esq." } else { "Dto." }) } else { String::new() };
            let mobile = rng.chance(0.6);
            (
                format!("{} {}, {number}{floor}", rng.pick(c.street_types), rng.pick(c.street_names)),
                format!("{cp4}-{} {city}", digits(rng, 3)),
                if mobile {
                    format!("+351 9{} {} {}", rng.pick(&["1", "2", "3", "6"]), digits(rng, 3), digits(rng, 3))
                } else {
                    let rest = 9 - area.len();
                    let d = digits(rng, rest);
                    format!("+351 {area}{} {}", &d[..rest - 6], &d[rest - 6..]).replace("  ", " ")
                },
            )
        }
        _ => {
            let floor = if rng.chance(0.5) { format!(", {}º {}", rng.range(1, 9), ["A", "B", "C", "D"][rng.below(4)]) } else { String::new() };
            let mobile = rng.chance(0.6);
            let phone = if mobile {
                format!("+34 6{} {} {} {}", digits(rng, 2), digits(rng, 2), digits(rng, 2), digits(rng, 2))
            } else {
                let d = format!("{area}{}", digits(rng, 9 - area.len()));
                format!("+34 {} {} {} {}", &d[..3], &d[3..5], &d[5..7], &d[7..])
            };
            (
                format!("{} {}, {number}{floor}", rng.pick(c.street_types), rng.pick(c.street_names)),
                format!("{prefix}{} {city}", digits(rng, 5 - prefix.len())),
                phone,
            )
        }
    };
    let email = email.then(|| {
        let domain = rng.pick(&["example.com", "example.org", "example.net"]);
        let last = surnames.last().copied().unwrap_or("");
        format!("{}.{}{}@{domain}", ascii(first), ascii(last), rng.range(1, 99))
    });
    Person { name, street, city_line, phone, email }
}

fn csv_field(s: &str) -> String {
    if s.contains([',', '"', '\n']) { format!("\"{}\"", s.replace('"', "\"\"")) } else { s.to_string() }
}

fn json_str(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

fn main() {
    let mut cli = Cli::new("rrig");
    let t = Lang::pick(cli.lang, &EN, &PT, &ES);
    let mut count = 1usize;
    let mut country: &Country = match cli.lang {
        Lang::Pt => &PORTUGAL,
        Lang::Es => &SPAIN,
        Lang::En => &US,
    };
    let (mut gender, mut email, mut format, mut seed) = (None::<bool>, false, "text".to_string(), None::<u64>);
    while let Some(a) = cli.next() {
        match a {
            Arg::Pos(p) => cli.unknown(&p),
            Arg::Opt(opt) => match opt.as_str() {
                "-h" | "--help" => cli.help(t.help),
                "-V" | "--version" => cli.version(),
                "-c" | "--count" => count = cli.number(&opt, 1..=1_000_000),
                "-C" | "--country" => {
                    let v = cli.value(&opt).to_ascii_lowercase();
                    country = COUNTRIES
                        .iter()
                        .find(|c| c.code == v)
                        .copied()
                        .unwrap_or_else(|| cli.fail(&format!("{} '{v}' (us, pt, es)", t.unknown_country)));
                }
                "-m" | "--male" => gender = Some(true),
                "-f" | "--female" => gender = Some(false),
                "-e" | "--email" => email = true,
                "-o" | "--format" => {
                    format = cli.value(&opt).to_ascii_lowercase();
                    if !matches!(format.as_str(), "text" | "csv" | "json") {
                        cli.fail(&format!("{} '{opt}': {format}", fun::i18n::common(cli.lang).invalid_value));
                    }
                }
                "-s" | "--seed" => seed = Some(cli.parse(&opt)),
                _ => cli.unknown(&opt),
            },
        }
    }
    let mut rng = seed.map_or_else(Rng::new, Rng::seeded);
    let mut out = String::new();
    if format == "csv" {
        out.push_str(if email { "name,street,city,phone,email\n" } else { "name,street,city,phone\n" });
    } else if format == "json" {
        out.push_str("[\n");
    }
    for i in 0..count {
        let p = person(country, &mut rng, gender, email);
        match format.as_str() {
            "csv" => {
                let mut f = vec![csv_field(&p.name), csv_field(&p.street), csv_field(&p.city_line), csv_field(&p.phone)];
                if let Some(e) = &p.email {
                    f.push(csv_field(e));
                }
                out.push_str(&f.join(","));
                out.push('\n');
            }
            "json" => {
                let mut f = vec![
                    format!("\"name\": {}", json_str(&p.name)),
                    format!("\"street\": {}", json_str(&p.street)),
                    format!("\"city\": {}", json_str(&p.city_line)),
                    format!("\"phone\": {}", json_str(&p.phone)),
                ];
                if let Some(e) = &p.email {
                    f.push(format!("\"email\": {}", json_str(e)));
                }
                out.push_str(&format!("  {{ {} }}{}\n", f.join(", "), if i + 1 < count { "," } else { "" }));
            }
            _ => {
                if i > 0 {
                    out.push('\n');
                }
                out.push_str(&format!("{}\n{}\n{}\n{}\n", p.name, p.street, p.city_line, p.phone));
                if let Some(e) = &p.email {
                    out.push_str(&format!("{e}\n"));
                }
            }
        }
    }
    if format == "json" {
        out.push_str("]\n");
    }
    use std::io::Write;
    let _ = std::io::stdout().write_all(out.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_country_makes_people() {
        let mut rng = Rng::seeded(3);
        for c in COUNTRIES {
            for _ in 0..200 {
                let p = person(c, &mut rng, None, true);
                assert!(p.name.split(' ').count() > c.surname_count);
                assert!(p.email.as_deref().is_some_and(|e| e.is_ascii() && e.contains("@example.")));
                assert!(!p.phone.contains("  "), "{}", p.phone);
            }
        }
        let us = person(&US, &mut rng, Some(false), false);
        assert!(us.phone.contains(") 555-01"));
        assert_eq!(csv_field("a,b"), "\"a,b\"");
        assert_eq!(json_str("a\"b"), "\"a\\\"b\"");
    }
}
