#![allow(clippy::cargo)]

use std::{collections::HashMap, fs, str::FromStr as _, sync::LazyLock, time::Duration};

use itertools::Itertools as _;
use latkerlo_jvotci::{
    Settings, analyze_brivla,
    katna::selrafsi_list_from_rafsi_list,
    tarmi::{BrivlaType, is_consonant},
};
use regex::Regex;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::Value;

#[allow(clippy::too_many_lines)]
#[allow(clippy::format_push_string)]
fn main() -> Result<(), ()> {
    let settings = Settings::from_str("A1rgz").unwrap();
    let client = Client::builder().timeout(Duration::from_secs(60)).build().unwrap();
    let jvs = client
        .get("https://github.com/mi2ebi/dictionary-counter/raw/refs/heads/master/jvs.txt")
        .send()
        .unwrap()
        .text()
        .unwrap();
    let jvs = jvs.lines().collect_vec();
    let mut tauste = vec![];
    let lidysisku = client
        .get("https://github.com/lynn/lidysisku/raw/refs/heads/gh-pages/jvs-en.json")
        .send()
        .unwrap()
        .text()
        .unwrap();
    let defs = serde_json::from_str::<Vec<Value>>(&lidysisku).unwrap();
    for word in jvs {
        if !is_consonant(word.chars().last().unwrap())
            && let Ok(tanru) = analyze_brivla(word, &settings)
        {
            let veljvo = selrafsi_list_from_rafsi_list(&tanru.1, &settings).unwrap();
            if [BrivlaType::ExtendedLujvo, BrivlaType::Lujvo].contains(&tanru.0)
                && !veljvo.iter().any(|valsi| valsi.contains('-'))
                && let Some(def) = defs.iter().find(|def| def[0] == word)
            {
                tauste.push((veljvo, word, def[4].as_str().unwrap()));
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
        .filter(|(_, n)| n >= &10)
        .collect_vec();
    let mut freqs_string = String::new();
    for freq in freqs {
        freqs_string += &format!("{}   {:?}\n", freq.0, freq.1);
    }
    fs::write("freqs.txt", freqs_string).unwrap();
    for (i, (tanru, _, _)) in tauste.clone().into_iter().enumerate() {
        if tanru.iter().all(|valsi| TOAQIZER.contains_key(&valsi.as_str())) {
            tauste[i].0 = tanru
                .iter()
                .enumerate()
                .map(|(j, valsi)| {
                    let toaqized =
                        TOAQIZER.get(&valsi.as_str()).map_or_else(String::new, ToString::to_string);
                    if toaqized.starts_with('\'') && j == 0 {
                        toaqized[1..].to_string()
                    } else {
                        toaqized
                    }
                })
                .collect();
        } else {
            tauste[i].0 = vec![];
        }
    }
    let toadua = client
        .get("https://github.com/mi2ebi/dictionary-counter/raw/refs/heads/master/toadua.txt")
        .send()
        .unwrap()
        .text()
        .unwrap();
    let metoame = tauste
        .iter()
        .filter(|(metoa, _, _)| {
            !metoa.is_empty() && !toadua.lines().any(|toa| toa == metoa.join(""))
        })
        .map(|(metoa, lujvo, def)| (metoa.join(""), lujvo, def))
        .collect_vec();
    let mut metoame_string = String::new();
    for (metoa, lujvo, def) in &metoame {
        metoame_string += &format!("{metoa}\t{lujvo}\t{def}\n");
    }
    fs::write("metoame.tsv", metoame_string).unwrap();
    println!("was able to toaqize \x1b[92m{}\x1b[m/{orig_len} lujvo", metoame.len());
    let nonletter = Regex::new(r"\W").unwrap();
    // rust moment
    let words = metoame
        .iter()
        .map(|(_, _, def)| def.to_lowercase())
        .collect_vec()
        .iter()
        .flat_map(|def| def.split([' ', '/', '-', ',', '=']).collect_vec())
        .map(|word| nonletter.replace_all(word, "").to_string())
        .sorted()
        .dedup()
        .collect_vec();
    println!("found \x1b[92m{}\x1b[m unique words in the lojban definitions", words.len());
    let toadua = client
        .post("https://toadua.uakci.space/api")
        .body(r#"{"action": "search", "query": ["scope", "en"]}"#)
        .send()
        .unwrap();
    if !toadua.status().is_success() {
        println!("\x1b[91mtoadua is down :< status code {}\x1b[m", toadua.status());
        return Err(());
    }
    let toadua = serde_json::from_reader::<_, Toadua>(toadua)
        .unwrap()
        .results
        .iter()
        .map(|toa| toa.body.to_lowercase())
        .collect_vec()
        .iter()
        .flat_map(|toa| toa.split([' ', '/', '-']).collect_vec())
        .map(|word| nonletter.replace_all(word, "").to_string())
        .collect_vec();
    let x_n = Regex::new(r"^[a-z]+_?\{?\d+\}?$").unwrap();
    let cmavrnu_liho = Regex::new("^(nu|ka|(se)?duu|sio|lue|lii?|zo|jou)$").unwrap();
    let ohno = words
        .iter()
        .filter(|word| {
            !toadua.contains(word) && !x_n.is_match(word) && !cmavrnu_liho.is_match(word)
        })
        .collect_vec();
    println!("\x1b[92m{}\x1b[m of them aren't in toadua", ohno.len());
    #[allow(clippy::literal_string_with_formatting_args)]
    let html = "<!doctype html><html><head>".to_string()
        + "<meta name='viewport' content='width=device-width,initial-scale=1'/>"
        + "<style>"
        + "html{font-family:'fira sans','noto sans','stix two text',serif}"
        + "a{color:blueviolet}"
        + "b{color:red}"
        + "th,td{text-align:left;vertical-align:top;padding-top:0.3lh}"
        + ".gray{color:gray}"
        + "math{font-family:'fira math','noto sans math','stix two math',math}"
        + "p:has(#nogray:checked)~table .gray{display:none}"
        + "@media(prefers-color-scheme:dark){"
        + "html{background:black;color:white}"
        + "b{color:orange}"
        + "a{color:turquoise}"
        + "}</style>"
        + "<script src='temml/dist/temml.min.js'></script>"
        + "</head>\n"
        + &format!("<body><h1>free calques of {} lujvo!</h1>\n", metoame.len())
        + "<p>"
        + "made with jbovlaste "
        + "(<a href='https://github.com/mi2ebi/dictionary-counter/blob/master/jvs.txt'>"
        + "indirectly</a>), "
        + "<a href='https://github.com/lynn/lidysisku'>lidysisku</a>, "
        + "<a href='https://github.com/toaq/toadua'>toadua</a>, "
        + "<a href='https://github.com/latkerlo/latkerlo-jvotci'>latkerlo-jvotci</a>, "
        + "<a href='https://github.com/mi2ebi/jvoaq/blob/master/src/main.rs#L228'>"
        + "a big hashmap</a>"
        + "<br/>"
        + "<input type='checkbox' id='nogray'/><label for='nogray'>hide gray entries</label></p>\n"
        + &format!(
            "<table>\n{}\n</table>",
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
                        if bolded.contains("<b>") || def.is_empty() { "" } else { " class='gray'" }
                    )
                })
                .join("\n")
        )
        + "<script>"
        + "temml.renderMathInElement(document.body,{delimiters:[{left:'$',right:'$'}]})"
        + "</script>"
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

static TOAQIZER: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    HashMap::from([
        ("bacru", "choa"),
        ("badri", "meo"),
        ("bajra", "jara"),
        ("bakni", "guobe"),
        ("balvi", "bıe"),
        // ("bancu", ""),
        ("bandu", "leoq"),
        ("bangu", "zu"),
        ("banli", "suoı"),
        ("banro", "jeaq"),
        ("banzu", "bıaq"),
        // ("bapli", "caıtua"),
        ("barda", "sao"),
        // ("barja", ""),
        // ("barna", ""),
        ("bartu", "buı"),
        ("basti", "dıba"),
        ("batci", "kaqga"),
        ("batke", "cıoq"),
        ("benji", "dıeq"),
        ("berti", "bero"),
        ("besna", "kera"),
        // ("betfu", ""),
        ("bevri", "hıe"),
        ("bi", "roaı"),
        ("bilma", "bıa"),
        ("binxo", "sho"),
        ("birka", "gıe"),
        ("bisli", "kıeı"),
        ("bitmu", "goeq"),
        ("blabi", "bao"),
        ("blanu", "mıo"),
        ("bliku", "gam"),
        ("bloti", "meaq"),
        // ("bo", ""),
        ("bolci", "kıoq"),
        ("bongu", "kuoq"),
        ("botpi", "cheoq"),
        ("boxfo", "boe"),
        ("boxna", "sueq"),
        ("bredi", "buo"),
        ("bridi", "jabı"),
        // ꝠAJUI BÁQ ZUDIUTOA MEOZUNO
        ("brife", "'ırue"),
        ("bukpu", "gueq"),
        ("burcu", "chuım"),
        ("cabna", "naı"),
        ("cabra", "kea"),
        ("cacra", "hora"),
        ("cadzu", "koı"),
        ("calku", "pıu"),
        ("cando", "dom"), // suo, dom
        // ("cange", ""),
        ("canko", "chuao"),
        ("canlu", "goa"),
        // ("carce", ""),
        ("carmi", "caı"),
        ("carna", "muoı"),
        ("carvi", "ruq"),
        ("casnu", "keoı"),
        ("catlu", "kaqsı"),
        ("catni", "cue"),
        ("catra", "jıam"),
        ("cecla", "cara"),
        // ("cecmu", "mıeme"),
        ("cenba", "beo"),
        ("certu", "joe"),
        ("cevni", "jıao"),
        ("cfari", "ceo"),
        ("cfika", "lua"),
        ("ci", "saq"),
        ("ciblu", "sıaı"),
        ("cidja", "haq"),
        ("cidni", "bea"),
        ("cilce", "puaı"),
        ("cilmo", "cuaı"),
        ("cilre", "chıe"),
        ("cindu", "heıga"),
        ("cinfo", "labı"),
        ("cinki", "chom"),
        ("cinmo", "moe"),
        ("cinri", "sıgı"),
        ("cinse", "seje"),
        ("cipni", "shuao"),
        ("cipra", "mıeq"),
        ("ciska", "kaı"),
        ("ciste", "doem"),
        ("citka", "chuq"),
        ("citno", "nıo"),
        // ("citri", "pudıu"),
        ("ckafi", "kafe"),
        ("ckaji", "'ıq"),
        // ("ckana", ""),
        ("ckape", "hıam"),
        ("ckini", "cuoı"),
        ("ckule", "chıejıo"),
        ("ckunu", "'ukomuao"),
        ("clani", "buaı"),
        ("claxu", "cıa"),
        ("cliva", "tıshaı"),
        ("cmalu", "nuı"),
        ("cmana", "meı"),
        ("cmavo", "doetoa"),
        ("cmene", "chua"),
        ("cmila", "hıaı"),
        ("cmima", "mea"),
        ("cmoni", "shoı"),
        ("cnebo", "boa"),
        ("cnino", "nıq"),
        ("cnita", "guq"),
        ("cortu", "noı"),
        ("cpacu", "nua"),
        ("cpedu", "sue"),
        ("crane", "shaq"),
        ("creka", "shatı"),
        ("crida", "'aıpı"),
        ("crino", "rıq"),
        ("cripu", "coa"),
        ("ctuca", "gale"),
        ("cukla", "feoq"),
        ("cukta", "kue"),
        ("cumki", "daı"),
        ("cuntu", "tue"),
        ("cupra", "shuaq"),
        ("curmi", "shoe"),
        ("cusku", "kuq"),
        ("cutci", "puefuq"),
        ("cuxna", "koe"),
        ("da", "raı"),
        ("dacti", "raı"),
        ("dakfu", "torea"),
        ("dakli", "cea"),
        ("damba", "soı"),
        ("dandu", "beaı"),
        ("danlu", "nıaı"),
        ("dansu", "marao"),
        ("dargu", "tıeq"),
        ("darno", "jao"),
        ("darxi", "dea"),
        ("dasri", "gıa"),
        ("datni", "dao"),
        ("degji", "cheı"),
        ("dekto", "heı"),
        ("denci", "nıoq"),
        ("denmi", "juıtaq"),
        ("denpa", "lao"),
        ("dertu", "'asaı"),
        ("detri", "daqchıu"),
        ("dikca", "ceoq"),
        ("dinju", "jıo"),
        ("djacu", "nao"),
        ("djedi", "chaq"),
        ("djica", "shao"),
        ("djine", "feoq"),
        ("djuno", "dua"),
        ("donri", "dıo"),
        ("drani", "due"),
        ("drata", "heo"),
        ("du", "jeq"),
        ("dukse", "duı"),
        ("dukti", "gıq"),
        ("dunda", "do"),
        ("dunli", "jeq"),
        // ("dzena", ""),
        ("facki", "gaı"), // ???
        ("fagri", "loe"),
        ("fanmo", "fao"),
        // ("fapro", ""),
        ("farlu", "shua"),
        ("farna", "feo"),
        ("farvi", "beo"),
        ("fasnu", "faq"),
        ("fatne", "'onuq"),
        // ("fatri", ""),
        ("fengu", "feı"),
        ("festi", "mute"),
        ("fetsi", "lıq"),
        ("finpe", "cıe"),
        ("finti", "fıeq"),
        ("flalu", "juao"),
        ("flecu", "hıu"),
        ("fliba", "buaq"),
        ("flira", "shom"),
        ("foldi", "dueq"),
        ("fonxa", "foq"),
        ("frati", "cua"),
        ("frica", "heo"),
        ("friko", "'afarı"),
        ("frili", "fuı"),
        ("fukpi", "kopı"),
        ("fuzme", "caq'eı"),
        ("gacri", "tıe"),
        ("galfi", "beo"),
        ("galtu", "gea"),
        ("ganse", "gaı"),
        ("gapci", "shına"),
        ("gapru", "gao"),
        ("gasnu", "tua"),
        ("gerku", "kune"),
        ("gerna", "zujuao"),
        ("girzu", "me"),
        ("glare", "loq"),
        ("gleki", "jaı"),
        ("gletu", "seaq"),
        ("glico", "'ıqlı"),
        ("grana", "beaq"),
        ("grasu", "nulı"),
        ("greku", "fuom"),
        ("grute", "zeo"),
        ("gubni", "cueq"),
        ("gugde", "gua"),
        // ("gundi", ""),
        ("gunka", "guaı"),
        ("gunma", "me"),
        ("gurni", "guı"),
        ("gusni", "gıo"),
        ("ja", "ra"),
        ("jadni", ""),
        // ("jai", ""),
        ("jalge", "se"),
        ("jamfu", "pue"),
        ("jamna", "soı"),
        ("jarco", "'ıjo"),
        ("jatna", "joaq"),
        ("javni", "juao"),
        ("jbari", "kurı"),
        ("jbena", "jıu"),
        ("jbini", "rıe"),
        ("jdari", "teınoa"),
        ("jdice", "koe"),
        ("jdika", "dıa"),
        ("jdini", "nuaı"),
        ("je", "ru"),
        // ("jecta", ""),
        ("jeftu", "joa"),
        ("jelca", "hoaq"),
        ("jemna", "lıem"),
        ("jetnu", "juna"),
        ("jgari", "jıaı"),
        ("jgina", "genea"),
        ("jibni", "juı"),
        ("jibri", "che"), // ?
        ("jicmu", "beoq"),
        ("jikca", "soao"),
        ("jimpe", "lım"),
        ("jinme", "loha"),
        ("jinru", "shoem"),
        ("jinvi", "mıu"),
        ("jipno", "jıeq"),
        ("jitfa", "sahu"),
        ("jitro", "caq"),
        ("jivna", "soqluaq"),
        ("jmive", "mıe"),
        ("joi", "roı"),
        ("jorne", "coe"),
        ("judri", "chıu"),
        ("jufra", "kune"),
        ("jukpa", "haqbaı"),
        ("julne", "koaı"),
        ("jundi", "sı"),
        // ("jutsi", ""),
        // ("ka", ""),
        // ("kagni", ""), // fıuq?
        ("kakne", "deq"),
        ("kalri", "rıa"),
        ("kanla", "kaq"),
        ("kansa", "gaq"),
        ("kantu", "toaı"),
        ("karce", "chao"),
        ("karda", "kata"),
        ("katna", "toe"),
        // ("ke", ""),
        ("kelci", "luaq"),
        ("kensa", "sheamı"),
        ("kerfa", "kıaq"),
        ("kerlo", "moma"),
        ("kevna", "jeoq"),
        ("kibro", "zıq"),
        ("kilto", "bıq"),
        ("klama", "fa"),
        ("klani", "nhe"),
        ("klesi", "rıoq"),
        ("korbi", "rea"),
        ("krasi", "sıao"),
        ("krati", "coq"),
        ("krefu", "guo"),
        ("krici", "chı"),
        ("krinu", "kuıca"),
        ("kruca", "chıeq"),
        ("kubli", "gam"),
        ("kulnu", "cıao"),
        ("kumfa", "kua"),
        ("kunra", "pıo"),
        ("kunti", "shea"),
        // ("kurfa", ""),
        ("kurji", "kıaı"),
        ("lacpu", "baga"),
        ("ladru", "noaı"),
        ("lamji", "leaq"),
        ("lanme", "hobı"),
        ("lanzu", "luo"),
        ("larcu", "lea"),
        ("lerfu", "laı"),
        ("lifri", "lıe"),
        ("lijda", ""),
        ("linji", "gıu"),
        ("lisri", "lua"),
        ("liste", "mekao"),
        ("litki", "leu"),
        ("litru", "fa"),
        ("logji", "lojı"),
        ("lojbo", "lojıbaq"),
        ("loldi", "deaq"),
        ("lujvo", "metoa"),
        ("lumci", "sıqja"),
        ("mabla", "huı"),
        ("makcu", "koaq"),
        ("makfa", "majı"),
        ("maksi", "mueı"),
        ("mamta", "mama"),
        ("mapku", "chea"),
        ("mapti", "tıao"),
        ("marce", "chao"),
        ("marji", "saı"),
        ("masno", "meoq"),
        ("masti", "jue"),
        ("mei", "lıaq"),
        ("melbi", "de"),
        ("menli", "moıchu"),
        ("merko", "'usona"),
        ("merli", "mıeq"),
        ("midju", "chu"),
        ("mikce", "goı"),
        ("milti", "mılı"),
        ("milxe", "tuao"),
        ("minde", "sue"),
        ("minji", "kea"),
        ("minra", "nuoq"),
        ("mintu", "jeq"),
        ("mipri", "shuı"),
        ("mitre", "meta"),
        ("mixre", "saıme"),
        ("mlana", "lıa"),
        ("mlatu", "kato"),
        ("mleca", "kuoı"),
        ("moi", "ko"),
        ("moklu", "buq"),
        ("morji", "moaq"),
        ("morna", "guoteı"),
        ("morsi", "muoq"),
        // ("mosra", ""),
        ("mrilu", "dıeq"),
        ("mu", "fe"),
        ("mudri", "muaosaı"),
        ("mulno", "muo"),
        ("munje", "jıaq"),
        ("mutce", "jaq"),
        ("muvdu", "gıam"),
        ("na", "bu"),
        ("na'e", "bu"),
        ("nakni", "naq"),
        ("nalci", "shoaı"),
        ("namcu", "zıu"),
        ("nanba", "nam"),
        ("nanca", "nıaq"),
        ("nanmu", "naq"),
        ("nazbi", "shıma"),
        ("nelci", "cho"),
        ("nenri", "nıe"),
        // ("ni", ""),
        ("nibli", "she"),
        ("nicte", "nuaq"),
        ("nimre", "kero"),
        ("ninmu", "lıq"),
        // ("nirna", ""),
        ("nitcu", "chıa"),
        ("no", "sıa"),
        ("no'e", "tuao"),
        ("nobli", "ruaı"),
        ("notci", "juo"),
        // ("nu", ""),
        ("nunmu", "zeq"),
        ("pa", "shı"),
        ("pacna", "zaı"),
        ("pagbu", "paq"),
        ("pagre", "peo"),
        ("pajni", "jıe"),
        ("palci", "cheom"),
        // ("panci", ""),
        // ("panra", ""),
        ("panzi", "fu"),
        ("pe'a", "'aı"),
        ("pelji", "peq"),
        ("pendo", "paı"),
        ("pensi", "moı"),
        ("pesxu", "dashı"),
        ("pezli", "nıuboe"),
        // ("pikta", ""),
        ("pilji", "reu"),
        ("pilno", "choq"),
        ("pinji", "peso"),
        ("pinta", "bore"),
        ("pixra", "fuaq"),
        // ("platu", ""),
        ("pleji", "teq"),
        ("plini", "pıanete"),
        ("plipe", "loma"),
        ("pluka", "pua"),
        ("pluta", "tıeq"),
        ("polje", "tıduja"),
        ("ponse", "bo"),
        ("porpi", "poaq"),
        ("porsi", "chue"),
        ("prali", "nhuq"),
        ("prami", "maı"),
        ("prenu", "poq"),
        ("preti", "teoq"),
        ("pruce", "case"),
        ("punji", "tıdo"),
        ("purci", "pu"),
        ("purmo", "puo"),
        // ("rafsi", ""),
        ("ralju", "joq"),
        ("rarna", "roa"),
        ("rango", "tuaı"),
        ("ratni", "'atom"),
        ("re", "gu"),
        ("rebla", "huoı"),
        ("rectu", "nueq"),
        ("remna", "req"),
        ("renro", "hıeq"),
        ("respa", "saoro"),
        ("rinka", "ca"),
        ("rirni", "pao"),
        ("ritli", "rıtı"),
        ("ro", "tu"),
        ("roi", "chıo"),
        ("rokci", "pıo"),
        ("rozgu", "barua"),
        // ("rupnu", ""),
        ("rutni", "baıse"),
        ("sakci", "supu"),
        ("sance", "laq"),
        ("sanga", "suaq"),
        ("sanji", "chıaq"),
        // ("sanmi", ""),
        // ("sarxe", ""),
        ("saske", "dıu"),
        ("savru", "noq"),
        ("sazri", "caq"),
        // ("se", ""),
        ("sefta", "rem"),
        ("selci", "toaı"),
        ("selfu", "lueq"),
        ("senta", "boepaq"),
        ("sepli", "poe"),
        ("sevzi", "taq"),
        ("sidbo", "sıo"),
        ("sidju", "soa"),
        ("simlu", "du"),
        ("simsa", "sıu"),
        ("simxu", "cheo"),
        ("sinma", "doı"),
        ("sinxa", "laı"),
        ("sipna", "nuo"),
        ("sisti", "shaı"),
        ("skami", "rom"),
        ("skari", "reo"),
        ("skicu", "juoı"),
        ("skori", "nhoq"),
        ("slaka", "raku"),
        ("slanu", "sonatani"),
        ("slilu", "furı"),
        ("sluni", "kepa"),
        ("smacu", "muse"),
        ("smuci", "sokum"),
        ("smuni", "mıu"),
        ("snanu", "namı"),
        ("snidu", "sekuq"),
        ("snime", "nıao"),
        ("so'i", "puı"), // teeeeechnically tıopuı
        ("solri", "hoe"),
        ("sonci", "soıche"),
        ("sorcu", "shuo"),
        ("sovda", "poaı"),
        ("spati", "nıu"),
        ("speni", "seo"),
        ("spisa", "hea"),
        ("spoja", "notuq"),
        ("sraji", "sheaq"),
        ("srana", "raq"),
        ("srasu", "poıba"),
        ("srera", "gom"),
        ("sruri", "rıe"),
        ("stani", "tanı"),
        ("stedu", "joqhua"),
        ("stidi", "dıe"),
        ("stuna", "hochu"),
        // ("stura", ""),
        ("stuzi", "rıaq"),
        ("sudga", "gıao"),
        ("sumji", "neu"),
        ("sumti", "aqmı"),
        ("sutra", "suaı"),
        ("tabno", "kabo"),
        ("tadji", "chase"),
        ("tamca", "tama"),
        ("tanbo", "toq"),
        ("tanxe", "tıaı"),
        ("tarbi", "deto"),
        ("tarci", "nuım"),
        ("tarmi", "teı"),
        ("tarti", "ruo"),
        ("tavla", "keoı"),
        ("taxfu", "fuq"),
        ("tcadu", "doaq"),
        ("tcana", "ce"),
        ("tcica", "cheu"),
        ("tcika", "daqmoa"),
        ("tcini", "tue"),
        ("tcita", "daocoe"),
        // ("te", ""),
        ("temci", "daq"),
        ("tenfa", "teu"),
        ("terpa", "tea"),
        ("tirna", "huo"),
        ("titla", "duao"),
        ("to'e", "gıq"),
        ("tonga", "laqtoaı"),
        ("tordu", "doq"),
        ("traji", "soq"),
        ("trene", "chue"),
        ("tricu", "muao"),
        ("trixe", "tıa"),
        ("troci", "leo"),
        ("tsali", "caı"),
        ("tsiju", "poaı"),
        ("tubnu", "bıu"),
        ("tugni", "mıujeq"),
        ("tumla", "dueq"),
        ("tuple", "shıaq"),
        ("turni", "cue"),
        ("tutci", "chuo"),
        // ("utka", ""), // ???
        ("vacri", "rıo"),
        ("vajni", "suao"),
        ("valsi", "toa"),
        ("vanci", "seum"),
        // ("vanju", ""),
        ("vasru", "heq"),
        ("vasxu", "ceu"),
        // ("ve", ""),
        ("vecnu", "teqdo"),
        ("verba", "deo"),
        ("vikmi", "cıu"),
        ("vimcu", "shata"),
        // ("vinji", ""),
        ("viska", "kaq"),
        // ("vlile", ""),
        ("vo", "jo"),
        ("vofli", "lıaı"),
        ("voksa", "choalaq"),
        ("vorme", "kıao"),
        ("vraga", "kuamı"),
        ("vreji", "reaq"),
        ("xabju", "bua"),
        ("xadba", "gıem"),
        ("xadni", "tuaı"),
        ("xamgu", "gı"),
        ("xamsi", "naomı"),
        ("xance", "muq"),
        ("xanri", "'aobı"),
        ("xarci", "muıq"),
        // ("xe", ""),
        ("xedja", "cheja"),
        ("xekri", "kuo"),
        ("xirma", "'eku"),
        ("xislu", "sıoq"),
        ("xlali", "huı"),
        ("xrani", "hıao"),
        // ("xriso", ""),
        ("xrula", "rua"),
        ("xruti", "rıu"),
        ("xukmi", "seao"),
        ("xunre", "kıa"),
        ("xusra", "ruaq"),
        ("zabna", "gı"),
        ("zanru", "gımıu"),
        ("zarci", "dıem"),
        ("zasti", "jıq"),
        ("zbasu", "baı"),
        ("zdani", "bua"),
        ("zdile", "luaı"),
        ("zekri", "jucıte"),
        ("zenba", "jeaq"),
        ("zgana", "sı"),
        ("zgike", "gıaq"),
        // ("zi'o", ""),
        ("zifre", "sheı"),
        ("zirpu", "loa"),
        ("zmadu", "huaq"),
        // ("zu'o", ""),
        ("zukte", "tao"),
        ("zunti", "bebaq"),
        ("zutse", "tuı"),
        ("zvati", "tı"),
    ])
});
