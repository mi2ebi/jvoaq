use itertools::Itertools;
use latkerlo_jvotci::{get_veljvo, tarmi::is_consonant};
use regex::Regex;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, fs, time::Duration};

fn main() -> Result<(), ()> {
    let jvs = fs::read_to_string("dictionary-counter/jvs.txt").unwrap();
    let jvs = jvs.lines().collect_vec();
    let mut tauste = vec![];
    let lidysisku = fs::read_to_string("lidysisku/jvs-en.json").unwrap();
    let defs = serde_json::from_str::<Vec<Value>>(&lidysisku).unwrap();
    for word in jvs {
        if !is_consonant(word.chars().last().unwrap()) {
            if let Ok(tanru) = get_veljvo(word) {
                if !tanru
                    .iter()
                    .any(|valsi| valsi.contains('-') || ["nu", "ka"].contains(&valsi.as_str()))
                {
                    if let Some(def) = defs.iter().find(|def| def[0] == word) {
                        tauste.push((tanru, word, def[4].as_str().unwrap()));
                    }
                }
            }
        }
    }
    let orig_len = tauste.len();
    let mut freqs = HashMap::new();
    for (tanru, _, _) in tauste.clone() {
        for valsi in tanru {
            if freqs.contains_key(&valsi) {
                freqs.insert(valsi.clone(), freqs.get(&valsi).unwrap() + 1);
            } else {
                freqs.insert(valsi, 1);
            }
        }
    }
    let freqs = freqs
        .iter()
        .map(|(valsi, n)| (valsi.clone(), *n))
        .sorted_by_key(|(valsi, _)| valsi.clone())
        .sorted_by_key(|(_, n)| -*n)
        .collect_vec();
    let mut freqs_string = String::new();
    for freq in freqs {
        freqs_string += &format!("{}   {:?}\r\n", freq.0, freq.1);
    }
    fs::write("freqs.txt", freqs_string).unwrap();
    // thanku lynn <3
    let toaqizer = HashMap::from([
        ("gasnu", "tua"),
        ("to'e", "gıq"),
        ("na'e", "bu"),
        ("prenu", "poq"),
        ("cusku", "kuq"),
        ("saske", "dıu"),
        ("valsi", "toa"),
        ("skami", "rom"),
        ("klama", "fa"),
        ("bangu", "zu"),
        ("rinka", "ca"),
        ("binxo", "sho"),
        ("cmalu", "nuı"),
        ("claxu", "cıa"),
        ("tutci", "chuo"),
        ("simxu", "cheo"),
        ("sevzi", "taq"),
        ("turni", "cue"),
        ("sance", "laq"),
        ("pa", "shı"),
        ("djacu", "nao"),
        ("barda", "sao"),
        ("kelci", "luaq"),
        ("pilno", "choq"),
        ("gugde", "guo"),
        ("temci", "daq"),
        ("lojbo", "lojıbaq"),
        ("selci", "toaı"),
        ("bloti", "meaq"),
        ("re", "gu"),
        ("spati", "nıu"),
        ("zukte", "tao"),
        ("prami", "maı"),
        ("djedi", "chaq"),
        ("pagbu", "paq"),
        ("ckaji", "ıq"),
        ("clani", "buaı"),
        ("zdani", "bua"),
        ("speni", "seo"),
        ("cabra", "kea"),
        ("tarmi", "teı"),
        ("ciste", "doeme"),
        ("djica", "shao"),
        ("kakne", "deq"),
        ("xance", "muq"),
        ("cidja", "haq"),
        ("mutce", "jaq"),
        ("xamgu", "gı"),
        ("bolci", "kıoq"),
        ("cinmo", "moe"),
        ("girzu", "me"),
        ("citka", "chuq"),
        ("fasnu", "faq"),
        ("gletu", "seaq"),
        ("masti", "jue"),
        ("purci", "pu"),
        ("stuzi", "rıaq"),
        ("cmene", "chua"),
        ("namcu", "zıu"),
        ("jmive", "mıe"),
        ("muvdu", "gıam"),
        ("na", "bu"),
        ("gusni", "gıo"),
        ("porsi", "chue"),
        ("grana", "beaq"),
        ("mabla", "huı"),
        ("sutra", "suaı"),
        ("carmi", "caı"),
        ("lerfu", "laı"),
        ("mei", "lıaq"),
        ("simsa", "sıu"),
        ("pensi", "moı"),
        ("kevna", "jeoq"),
        ("taxfu", "fuq"),
        ("marce", "chao"),
        ("mintu", "jeq"),
        ("morsi", "muoq"),
        ("so'i", "puı"),
        ("tumla", "dueq"),
        ("cukta", "kue"),
        ("dukse", "duı"),
        ("pixra", "fuaq"),
        ("tavla", "keoı"),
        ("vreji", "reaq"),
        ("zbasu", "baı"),
        ("cnita", "guq"),
        ("kerfa", "kıaq"),
        ("klesi", "rıoq"),
        ("lamji", "leaq"),
        ("rokci", "pıo"),
        ("bongu", "kuoq"),
        ("jibni", "juı"),
        ("jitro", "caq"),
        ("vasxu", "ceu"),
        ("djuno", "dua"),
        ("nobli", "ruaı"),
        ("bacru", "choa"),
        ("bakni", "guobe"),
        ("cmima", "mea"),
        ("ganse", "gaı"),
        ("gleki", "jaı"),
        ("milxe", "tuao"),
        ("minji", "kea"),
        ("sinxa", "laı"),
        ("vasru", "heq"),
        ("balvi", "bıe"),
        ("bukpu", "gueq"),
        ("danlu", "nıaı"),
        ("degji", "cheı"),
        ("mulno", "muo"),
        ("skari", "reo"),
        ("certu", "joe"),
        ("darno", "jao"),
        ("dinju", "jıo"),
        ("dunli", "jeq"),
        ("jorne", "coe"),
        ("melbi", "de"),
        ("morji", "moaq"),
        ("nelci", "cho"),
        ("pajni", "jıe"),
        ("smuni", "mıu"),
        ("cadzu", "koı"),
        ("dikca", "ceoq"),
        ("je", "ru"),
        ("notci", "juo"),
        ("tricu", "muao"),
        ("xukmi", "seao"),
        ("zgike", "gıaq"),
        ("carna", "muoı"),
        ("cupra", "shuaq"),
        ("finpe", "cıe"),
        ("jinme", "loha"),
        ("kumfa", "kua"),
        ("midju", "chu"),
        ("remna", "req"),
        ("ro", "tu"),
        ("vorme", "kıao"),
        ("xadni", "tuaı"),
        ("cukla", "moem"),
        ("fetsi", "lıq"),
        ("kansa", "gaq"),
        ("lifri", "lıe"),
        ("lumci", "sıqja"),
        ("sidbo", "sıo"),
        ("sisti", "shaı"),
        ("ci", "saq"),
        ("fagri", "loe"),
        ("kantu", "toaı"),
        ("zenba", "jeaq"),
        ("bartu", "buı"),
        ("citno", "nıo"),
        ("galtu", "gea"),
        ("glare", "loq"),
        ("jdini", "nuaı"),
        ("jukpa", "haqbaı"),
        ("tarti", "ruo"),
        ("viska", "kaq"),
        ("cabna", "naı"),
        ("calku", "pıu"),
        ("denci", "nıoq"),
        ("flecu", "hıu"),
        ("katna", "toe"),
        ("moklu", "buq"),
        ("nakni", "naq"),
        ("nicte", "nuaq"),
        ("ponse", "bo"),
        ("trixe", "tıa"),
        ("banli", "suoı"),
        ("cipni", "shuao"),
        ("cpedu", "sue"),
        ("jatna", "joaq"),
        ("jetnu", "juna"),
        ("jikca", "soao"),
        ("krefu", "guo"),
        ("nanca", "nıaq"),
        ("pelji", "peq"),
        ("terpa", "tea"),
        ("zabna", "gı"),
        ("zgana", "sı"),
        ("zvati", "tı"),
        ("bevri", "hıe"),
        ("ciska", "kaı"),
        ("cnino", "nıq"),
        ("finti", "fıeq"),
        ("jbena", "jıu"),
        ("kalri", "rıa"),
        ("zifre", "sheı"),
        ("cevni", "jıao"),
        ("ckafi", "kafe"),
        ("crane", "shaq"),
        ("cuxna", "koe"),
        ("damba", "soı"),
        ("donri", "dıo"),
        ("drani", "due"),
        ("flalu", "juao"),
        ("gunma", "me"),
        ("lanzu", "luo"),
        ("marji", "saı"),
        ("menli", "moıchu"),
        ("mitre", "meta"),
        ("pagre", "peo"),
        ("plini", "pıanete"),
        ("sefta", "rem"),
        ("solri", "hoe"),
        ("sorcu", "shuo"),
        ("sruri", "rıe"),
        ("tubnu", "bıu"),
        ("xlali", "huı"),
        ("xusra", "ruaq"),
        ("bilma", "bıa"),
        ("cfari", "ceo"),
        ("cortu", "noı"),
        ("cpacu", "nua"),
        ("jinvi", "mıu"),
        ("joi", "roı"),
        ("karce", "chao"),
        ("larcu", "lea"),
        ("linji", "gıu"),
        ("logji", "lojı"),
        ("vo", "jo"),
        ("xabju", "bua"),
        ("farlu", "shua"),
        ("galfi", "beo"),
        ("gerna", "zujuao"),
        ("gunka", "guaı"),
        ("minde", "sue"),
        ("mudri", "muaosaı"),
        ("penmi", "geq"),
        ("plipe", "loma"),
        ("pluta", "tıeq"),
        ("tsiju", "poaı"),
        ("vimcu", "shata"),
        ("xamsi", "naomı"),
        ("casnu", "keoı"),
        ("cenba", "beo"),
        ("dakfu", "torea"),
        ("gubni", "cueq"),
        ("krici", "chı"),
        ("merli", "mıeq"),
        ("nanba", "nam"),
        ("sanga", "suaq"),
        ("selfu", "lueq"),
        ("sipna", "nuo"),
        ("xrani", "hıao"),
        ("birka", "gıe"),
        ("bitmu", "goeq"),
        ("ctuca", "gale"),
        ("cuntu", "tue"),
        ("dertu", "asaı"),
        ("farna", "feo"),
        ("glico", "ıqlı"),
        ("jamfu", "pue"),
        ("jinru", "shoem"),
        ("jipno", "jıeq"),
        ("nenri", "nıe"),
        ("pendo", "paı"),
        ("rirni", "pao"),
        ("sazri", "caq"),
        ("sepli", "poe"),
        ("spoja", "notuq"),
        ("stedu", "joqhua"),
        ("tanbo", "toq"),
        ("xrula", "rua"),
        ("curmi", "shoe"),
        ("cutci", "puefuq"),
        ("dandu", "beaı"),
        ("darxi", "dea"),
        ("datni", "dao"),
        ("denpa", "lao"),
        ("drata", "heo"),
        ("du", "jeq"),
        ("festi", "mute"),
        ("fukpi", "kopı"),
        ("gerku", "kune"),
        ("grute", "zeo"),
        ("jgari", "jıaı"),
        ("jicmu", "beoq"),
        ("kanla", "kaq"),
        ("kilto", "bıq"),
        ("krasi", "sıao"),
        ("kulnu", "cıao"),
        ("mlana", "lıa"),
        ("munje", "jıaq"),
        ("pluka", "pua"),
        ("preti", "teoq"),
        ("ritli", "rıtı"),
        ("simlu", "du"),
        ("skori", "nhoq"),
        ("sonci", "soıche"),
        ("sraji", "sheaq"),
        ("tsali", "caı"),
        ("xalka", "seaı"),
        ("xislu", "sıoq"),
        ("benji", "dıeq"),
        ("canlu", "goa"),
        ("carvi", "ruq"),
        ("cumki", "daı"),
        ("fanmo", "fao"),
        ("fliba", "buaq"),
        ("korbi", "rea"),
        ("kruca", "chıeq"),
        ("litki", "leu"),
        ("loldi", "deaq"),
        ("rectu", "nueq"),
        ("renro", "hıeq"),
        ("srana", "raq"),
        ("tcica", "cheu"),
        ("tordu", "doq"),
        ("vajni", "suao"),
        ("xirma", "eku"),
        ("xruti", "rıu"),
        ("xunre", "kıa"),
        ("zutse", "tuı"),
        ("cilmo", "cuaı"),
        ("cladu", "laqcaı"),
        ("cliva", "tıshaı"),
        ("crida", "aıpı"),
        ("crino", "rıq"),
        ("dargu", "tıeq"),
        ("dasri", "gıa"),
        ("dunda", "do"),
        ("foldi", "dueq"),
        ("jamna", "soı"),
        ("jundi", "sı"),
        ("mapti", "tıao"),
        ("sidju", "soa"),
        ("skicu", "juoı"),
        ("smuci", "sokum"),
        ("snanu", "namı"),
        ("sudga", "gıao"),
        ("tcadu", "doaq"),
        ("vikmi", "cıu"),
        ("vofli", "lıaı"),
        ("blabi", "bao"),
        ("cfika", "lua"),
        ("cnebo", "boa"),
        ("dansu", "marao"),
        ("makfa", "majı"),
        ("savru", "noq"),
        ("tirna", "huo"),
        ("vacri", "rıo"),
        ("voksa", "choalaq"),
        ("cmana", "meı"),
        ("cmoni", "shoı"),
        ("frili", "fuı"),
        ("javni", "juao"),
        ("jbini", "rıe"),
        ("jimpe", "lım"),
        ("jufra", "kuna"),
        ("nitcu", "chıa"),
        ("tanxe", "tıaı"),
        ("trene", "chue"),
        ("tuple", "shıaq"),
        ("verba", "deo"),
        ("bajra", "jara"),
        ("cilce", "puaı"),
        ("creka", "shatı"),
        ("friko", "afarı"),
        ("ladru", "noaı"),
        ("lisri", "lua"),
        ("makcu", "koaq"),
        ("snime", "nıao"),
        ("tcika", "daqmoa"),
        ("badri", "meo"),
        ("catlu", "kaqsı"),
        ("ciblu", "sıaı"),
        ("fengu", "feı"),
        ("gapru", "gao"),
        ("jeftu", "joa"),
        ("kibro", "zıq"),
        ("rarna", "roa"),
        ("srasu", "poıba"),
        ("stuna", "hochu"),
        ("troci", "leo"),
        ("zarci", "dıem"),
    ]);
    for (i, (tanru, _, _)) in tauste.clone().into_iter().enumerate() {
        if tanru
            .iter()
            .all(|valsi| toaqizer.contains_key(&valsi.as_str()))
        {
            tauste[i].0 = tanru
                .iter()
                .map(|valsi| {
                    toaqizer.get(&valsi.as_str()).map_or_else(String::new, ToString::to_string)
                })
                .collect();
        } else {
            tauste[i].0 = vec![];
        }
    }
    let metoame = tauste
        .iter()
        .filter(|(metoa, _, _)| {
            !metoa.is_empty()
                && !fs::read_to_string("dictionary-counter/toadua.txt")
                    .unwrap()
                    .lines()
                    .any(|toa| toa == metoa.join(""))
        })
        .map(|(metoa, lujvo, def)| (metoa.join(""), lujvo, def))
        .collect_vec();
    let mut metoame_string = String::new();
    for (metoa, lujvo, def) in metoame.clone() {
        metoame_string += &format!("{metoa}\t{lujvo}\t{def}\r\n");
    }
    fs::write("metoame.tsv", metoame_string).unwrap();
    println!(
        "was able to toaqize \x1b[92m{}\x1b[m/{orig_len} lujvo",
        metoame.len()
    );
    let nonletter = Regex::new(r"\W").unwrap();
    // rust moment
    let words = metoame
        .iter()
        .map(|(_, _, def)| def.to_lowercase())
        .collect_vec()
        .iter()
        .flat_map(|def| def.split([' ', '/']).collect_vec())
        .filter(|word| !word.contains('$'))
        .map(|word| nonletter.replace_all(word, "").to_string())
        .sorted()
        .dedup()
        .collect_vec();
    println!(
        "found \x1b[92m{}\x1b[m unique words in the lojban definitions",
        words.len()
    );
    let client = Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .unwrap();
    let toadua = client
        .post("https://toadua.uakci.space/api")
        .body(r#"{"action": "search", "query": ["scope", "en"]}"#)
        .send()
        .unwrap();
    if !toadua.status().is_success() {
        println!(
            "\x1b[91mtoadua is down :< status code {}\x1b[m",
            toadua.status()
        );
        return Err(());
    }
    let toadua = serde_json::from_reader::<_, Toadua>(toadua)
        .unwrap()
        .results
        .iter()
        .map(|toa| toa.body.clone().to_lowercase())
        .collect_vec()
        .iter()
        .flat_map(|toa| toa.split([' ', '/']).collect_vec())
        .map(|word| nonletter.replace_all(word, "").to_string())
        .collect_vec();
    let ohno = words
        .iter()
        .filter(|word| !toadua.contains(word))
        .collect_vec();
    println!("\x1b[92m{}\x1b[m of them aren't in toadua", ohno.len());
    // why does rustfmt do this so weirdly
    let html = "<!doctype html><html><head><meta \
                name=\"viewport\"content=\"width=device-width,initial-scale=1\"/><style>b{color:\
                red;}th,td{text-align:left;vertical-align:top;padding-top: \
                0.3lh;}.gray{color:gray;}@media(prefers-color-scheme:dark){html{background:black;\
                color:white;}b{color:orange;}}</style></head>\r\n"
        .to_string()
        + &format!(
            "<body><h1>free calques of {} lujvo :3</h1><table>\r\n{}\r\n</table>",
            metoame.len(),
            metoame
                .iter()
                .map(|(metoa, lujvo, def)| {
                    let bolded = def
                        .split(' ')
                        .map(|word| {
                            word.split('/')
                                .map(|word2| {
                                    if !word2.contains('$')
                                        && ohno.contains(
                                            &&nonletter.replace_all(word2, "").to_string(),
                                        )
                                    {
                                        format!("<b>{word2}</b>")
                                    } else {
                                        word2.to_string()
                                    }
                                })
                                .join("/")
                        })
                        .join(" ");
                    format!(
                        "<tr{}><th>{metoa}</th><td>{lujvo}</td><td>{bolded}</td></tr>",
                        if bolded.contains("<b>") {
                            ""
                        } else {
                            r#" class="gray""#
                        }
                    )
                })
                .join("\r\n")
        )
        + "</body></html>";
    fs::write("index.html", html).unwrap();
    Ok(())
}

// saddest structs in the world
#[derive(Deserialize)]
struct Toadua {
    results: Vec<Toa>,
}
#[derive(Deserialize)]
struct Toa {
    body: String,
}
