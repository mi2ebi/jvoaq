use itertools::Itertools;
use latkerlo_jvotci::{
    analyze_brivla,
    katna::selrafsi_list_from_rafsi_list,
    tarmi::{is_consonant, BrivlaType},
    Settings,
};
use regex::Regex;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, fs, str::FromStr, sync::LazyLock, time::Duration};

#[allow(clippy::too_many_lines)]
fn main() -> Result<(), ()> {
    let settings = Settings::from_str("A1rgz").unwrap();
    let jvs = fs::read_to_string("dictionary-counter/jvs.txt").unwrap();
    let jvs = jvs.lines().collect_vec();
    let mut tauste = vec![];
    let lidysisku = fs::read_to_string("lidysisku/jvs-en.json").unwrap();
    let defs = serde_json::from_str::<Vec<Value>>(&lidysisku).unwrap();
    for word in jvs {
        if !is_consonant(word.chars().last().unwrap()) {
            if let Ok(tanru) = analyze_brivla(word, &settings) {
                let veljvo = selrafsi_list_from_rafsi_list(&tanru.1, &settings).unwrap();
                if [BrivlaType::ExtendedLujvo, BrivlaType::Lujvo].contains(&tanru.0)
                    && !veljvo.iter().any(|valsi| valsi.contains('-'))
                {
                    if let Some(def) = defs.iter().find(|def| def[0] == word) {
                        tauste.push((veljvo, word, def[4].as_str().unwrap()));
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
        .filter(|(_, n)| n >= &10)
        .collect_vec();
    let mut freqs_string = String::new();
    for freq in freqs {
        freqs_string += &format!("{}   {:?}\r\n", freq.0, freq.1);
    }
    fs::write("freqs.txt", freqs_string).unwrap();
    for (i, (tanru, _, _)) in tauste.clone().into_iter().enumerate() {
        if tanru
            .iter()
            .all(|valsi| TOAQIZER.contains_key(&valsi.as_str()))
        {
            tauste[i].0 = tanru
                .iter()
                .map(|valsi| {
                    TOAQIZER
                        .get(&valsi.as_str())
                        .map_or_else(String::new, ToString::to_string)
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
        // ("banro", ""),
        ("banzu", "bıaq"),
        // ("bapli", ""),
        ("barda", "sao"),
        // ("barja", ""),
        // ("barna", ""),
        ("bartu", "buı"),
        ("basti", "dıba"),
        // ("batci", "kaqga"),
        ("batke", "cıoq"),
        ("benji", "dıeq"),
        ("berti", "bero"),
        // ("besna", ""),
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
        // ("bridi", ""), // ꝠAJUI BÁQ ZUDIUTOA MEOZUNO
        ("brife", "ırue"),
        ("bukpu", "gueq"),
        ("burcu", "chuım"),
        ("cabna", "naı"),
        ("cabra", "kea"),
        // ("cacra", ""),
        ("cadzu", "koı"),
        ("calku", "pıu"),
        // ("cando", ""),
        // ("cange", ""),
        ("canko", "chuao"),
        ("canlu", "goa"),
        // ("carce", ""),
        ("carmi", "caı"),
        ("carna", "muoı"),
        ("carvi", "ruq"),
        ("casnu", "keoı"),
        ("catlu", "kaqsı"),
        // ("catni", ""),
        ("catra", "jıam"),
        ("cecla", "cara"),
        // ("cecmu", ""),
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
        // ("cindu", ""),
        // ("cinfo", "labı"),
        ("cinki", "chom"),
        ("cinmo", "moe"),
        // ("cinri", "sıgı"),
        ("cinse", "seje"),
        ("cipni", "shuao"),
        // ("cipra", ""),
        ("ciska", "kaı"),
        ("ciste", "doem"),
        ("citka", "chuq"),
        ("citno", "nıo"),
        // ("citri", ""),
        ("ckafi", "kafe"),
        ("ckaji", "ıq"),
        // ("ckana", ""),
        ("ckape", "hıam"),
        // ("ckini", ""), // toı? cuoı?
        // ("ckule", ""),
        // ("ckunu", ""),
        ("clani", "buaı"),
        ("claxu", "cıa"),
        ("cliva", "tıshaı"),
        ("cmalu", "nuı"),
        ("cmana", "meı"),
        // ("cmavo", ""),
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
        ("crida", "aıpı"),
        ("crino", "rıq"),
        ("cripu", "coa"),
        ("ctuca", "gale"),
        ("cukla", "moem"),
        ("cukta", "kue"),
        ("cumki", "daı"),
        ("cuntu", "tue"),
        ("cupra", "shuaq"),
        ("curmi", "shoe"),
        ("cusku", "kuq"),
        ("cutci", "puefuq"),
        ("cuxna", "koe"),
        // ("da", ""),
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
        // ("dekto", ""),
        ("denci", "nıoq"),
        // ("denmi", ""),
        ("denpa", "lao"),
        ("dertu", "asaı"),
        // ("detri", ""),
        ("dikca", "ceoq"),
        ("dinju", "jıo"),
        ("djacu", "nao"),
        ("djedi", "chaq"),
        ("djica", "shao"),
        // ("djine", ""), // feoq?
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
        // ("facki", ""), // ꝡaqdo??????
        ("fagri", "loe"),
        ("fanmo", "fao"),
        // ("fapro", ""),
        ("farlu", "shua"),
        ("farna", "feo"),
        // ("farvi", ""),
        ("fasnu", "faq"),
        // ("fatne", ""),
        // ("fatri", ""),
        ("fengu", "feı"),
        ("festi", "mute"),
        ("fetsi", "lıq"),
        ("finpe", "cıe"),
        ("finti", "fıeq"),
        ("flalu", "juao"),
        ("flecu", "hıu"),
        ("fliba", "buaq"),
        // ("flira", ""),
        ("foldi", "dueq"),
        ("fonxa", "foq"),
        ("frati", "cua"),
        // ("frica", ""), // also heo?
        ("friko", "afarı"),
        ("frili", "fuı"),
        ("fukpi", "kopı"),
        // ("fuzme", ""),
        ("gacri", "tıe"),
        ("galfi", "beo"),
        ("galtu", "gea"),
        ("ganse", "gaı"),
        // ("gapci", ""),
        ("gapru", "gao"),
        ("gasnu", "tua"),
        ("gerku", "kune"),
        ("gerna", "zujuao"),
        ("girzu", "me"),
        ("glare", "loq"),
        ("gleki", "jaı"),
        ("gletu", "seaq"),
        ("glico", "ıqlı"),
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
        // ("jarco", "ıjo"),
        ("jatna", "joaq"),
        ("javni", "juao"),
        ("jbari", "kurı"),
        ("jbena", "jıu"),
        ("jbini", "rıe"),
        // ("jdari", ""),
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
        // ("jgina", "genea"),
        ("jibni", "juı"),
        // ("jibri", ""),
        ("jicmu", "beoq"),
        ("jikca", "soao"),
        ("jimpe", "lım"),
        ("jinme", "loha"),
        ("jinru", "shoem"),
        ("jinvi", "mıu"),
        ("jipno", "jıeq"),
        ("jitfa", "sahu"),
        ("jitro", "caq"),
        // ("jivna", ""),
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
        // ("karda", "kata"),
        ("katna", "toe"),
        // ("ke", ""),
        ("kelci", "luaq"),
        ("kensa", "sheamı"),
        ("kerfa", "kıaq"),
        // ("kerlo", ""),
        ("kevna", "jeoq"),
        ("kibro", "zıq"),
        ("kilto", "bıq"),
        ("klama", "fa"),
        // ("klani", ""),
        ("klesi", "rıoq"),
        ("korbi", "rea"),
        ("krasi", "sıao"),
        ("krati", "coq"),
        ("krefu", "guo"),
        ("krici", "chı"),
        // ("krinu", ""),
        ("kruca", "chıeq"),
        ("kubli", "gam"),
        ("kulnu", "cıao"),
        ("kumfa", "kua"),
        // ("kunra", ""),
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
        // ("liste", ""),
        ("litki", "leu"),
        ("litru", "fa"),
        ("logji", "lojı"),
        ("lojbo", "lojıbaq"),
        ("loldi", "deaq"),
        // ("lujvo", ""),
        ("lumci", "sıqja"),
        ("mabla", "huı"),
        ("makcu", "koaq"),
        ("makfa", "majı"),
        ("maksi", "mueı"),
        // ("mamta", ""),
        ("mapku", "chea"),
        ("mapti", "tıao"),
        ("marce", "chao"),
        ("marji", "saı"),
        ("masno", "meoq"),
        ("masti", "jue"),
        ("mei", "lıaq"),
        ("melbi", "de"),
        ("menli", "moıchu"),
        ("merko", "usona"),
        ("merli", "mıeq"),
        ("midju", "chu"),
        ("mikce", "goı"),
        // ("milti", ""),
        ("milxe", "tuao"),
        ("minde", "sue"),
        ("minji", "kea"),
        ("minra", "nuoq"),
        ("mintu", "jeq"),
        ("mipri", "shuı"),
        ("mitre", "meta"),
        // ("mixre", ""),
        ("mlana", "lıa"),
        ("mlatu", "kato"),
        ("mleca", "kuoı"),
        ("moi", "ko"),
        ("moklu", "buq"),
        ("morji", "moaq"),
        // ("morna", ""),
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
        // ("nalci", ""),
        ("namcu", "zıu"),
        ("nanba", "nam"),
        ("nanca", "nıaq"),
        // ("nazbi", ""),
        ("nelci", "cho"),
        ("nenri", "nıe"),
        // ("ni", ""),
        // ("nibli", ""),
        ("nicte", "nuaq"),
        ("nimre", "kero"),
        // ("ninmu", ""),
        // ("nirna", ""),
        ("nitcu", "chıa"),
        ("no", "sıa"),
        ("no'e", "tuao"),
        ("nobli", "ruaı"),
        ("notci", "juo"),
        // ("nu", ""),
        ("pa", "shı"),
        ("pacna", "zaı"),
        ("pagbu", "paq"),
        ("pagre", "peo"),
        ("pajni", "jıe"),
        ("palci", "cheom"),
        // ("panci", ""),
        // ("panra", ""),
        ("panzi", "fu"),
        ("pe'a", "aı"),
        ("pelji", "peq"),
        ("pendo", "paı"),
        ("pensi", "moı"),
        // ("pesxu", "dashı"),
        // ("pezli", "nıuboe"),
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
        // ("polje", ""), // tıdu?
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
        // ("rango", ""), // tuaı????
        ("ratni", "atom"),
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
        // ("roi", ""),
        ("rokci", "pıo"),
        // ("rozgu", ""),
        // ("rupnu", ""),
        ("rutni", "baıse"),
        // ("sakci", ""),
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
        // ("senta", ""),
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
        // ("slanu", ""),
        ("slilu", "furı"),
        // ("sluni", ""),
        // ("smacu", ""),
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
        // ("sumti", ""),
        ("sutra", "suaı"),
        ("tabno", "kabo"),
        ("tadji", "chase"),
        ("tamca", "tama"),
        ("tanbo", "toq"),
        ("tanxe", "tıaı"),
        // ("tarbi", ""),
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
        // ("tcita", ""),
        // ("te", ""),
        ("temci", "daq"),
        ("tenfa", "teu"),
        ("terpa", "tea"),
        ("tirna", "huo"),
        ("titla", "duao"),
        ("to'e", "gıq"),
        // ("tonga", ""),
        ("tordu", "doq"),
        ("traji", "soq"),
        ("trene", "chue"),
        ("tricu", "muao"),
        ("trixe", "tıa"),
        ("troci", "leo"),
        ("tsali", "caı"),
        ("tsiju", "poaı"),
        ("tubnu", "bıu"),
        // ("tugni", ""),
        ("tumla", "dueq"),
        ("tuple", "shıaq"),
        ("turni", "cue"),
        ("tutci", "chuo"),
        // ("utka", ""), // ???
        ("vacri", "rıo"),
        ("vajni", "suao"),
        ("valsi", "toa"),
        // ("vanci", ""),
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
        // ("vraga", ""),
        ("vreji", "reaq"),
        ("xabju", "bua"),
        // ("xadba", ""),
        ("xadni", "tuaı"),
        ("xamgu", "gı"),
        ("xamsi", "naomı"),
        ("xance", "muq"),
        ("xanri", "aobı"),
        ("xarci", "muıq"),
        // ("xe", ""),
        // ("xedja", ""),
        ("xekri", "kuo"),
        ("xirma", "eku"),
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
        // ("zanru", ""),
        ("zarci", "dıem"),
        ("zasti", "jıq"),
        ("zbasu", "baı"),
        ("zdani", "bua"),
        ("zdile", "luaı"),
        // ("zekri", ""),
        ("zenba", "jeaq"),
        ("zgana", "sı"),
        ("zgike", "gıaq"),
        // ("zi'o", ""),
        ("zifre", "sheı"),
        ("zirpu", "loa"),
        ("zmadu", "huaq"),
        // ("zu'o", ""),
        ("zukte", "tao"),
        // ("zunti", ""),
        ("zutse", "tuı"),
        ("zvati", "tı"),
    ])
});
