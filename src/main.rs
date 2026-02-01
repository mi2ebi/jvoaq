// this code is horrible. beware

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
    let client = Client::builder().timeout(Duration::from_mins(1)).build().unwrap();
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
                freqs.insert(valsi.clone(), &freqs[&valsi] + 1);
            } else {
                freqs.insert(valsi, 1);
            }
        }
    }
    let freqs = freqs
        .iter()
        .map(|(valsi, n)| (valsi.clone(), *n))
        .sorted_by_key(|(valsi, _)| valsi.clone())
        .filter(|(_, n)| *n >= 2)
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
    let cmavrnu_liho = Regex::new(
        "^(nu|ka|(se)?duu|sio|lue|lii?|zo|jou|zei|kee|sumti|cmavo|lujvo|zievla|fuivla)$",
    )
    .unwrap();
    let ohno = words
        .iter()
        .filter(|word| {
            !toadua.contains(word) && !x_n.is_match(word) && !cmavrnu_liho.is_match(word)
        })
        .collect_vec();
    println!("\x1b[92m{}\x1b[m of them aren't in toadua", ohno.len());
    #[allow(clippy::literal_string_with_formatting_args, clippy::uninlined_format_args)]
    let html = "<!doctype html><html><head>".to_string()
        + "<meta name='viewport' content='width=device-width,initial-scale=1'/>"
        + "<style>"
        + "*{-webkit-text-size-adjust:100%}"
        + "html{font-family:'fira sans','noto sans','stix two text',serif}"
        + "a{color:blueviolet}"
        + "b{color:red}"
        + "th,td{text-align:left;vertical-align:top;padding-top:0.3lh}"
        + ".gray{opacity:50%}"
        + "math{font-family:'fira math','noto sans math','stix two math',math}"
        + "p:has(#nogray:checked)~table .gray{display:none}"
        + "@media(prefers-color-scheme:dark){"
        + "html{background:black;color:white}"
        + "b{color:orange}"
        + "a{color:turquoise}"
        + ".gray{opacity:75%}"
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
        + "<a href='https://github.com/mi2ebi/jvoaq/blob/master/src/main.rs#L234'>"
        + "a big hashmap</a>"
        + "<br/>"
        + "<input type='checkbox' id='nogray'/>"
        + "<label for='nogray'>hide gray entries (might be slow)</label>"
        + "</p>\n"
        + "<table>\n"
        + &metoame
            .iter()
            .map(|(metoa, lujvo, def)| {
                let bolded = def
                    .split(' ')
                    .map(|word| {
                        word.split('/')
                            .map(|word2| {
                                if !word2.contains('$')
                                    && ohno.contains(&&nonletter.replace_all(word2, "").to_string())
                                {
                                    format!("<b>{word2}</b>")
                                } else {
                                    word2.to_string()
                                }
                            })
                            .join("/")
                    })
                    .join(" ");
                "<tr".to_string()
                    + if bolded.contains("<b>") || def.is_empty() { ">" } else { " class='gray'>" }
                    + &format!("<th>{metoa}</th>")
                    + "<td>"
                    + &format!("<a href=\"https://xlasisku.github.io/?q={}\">{0}</a>", lujvo)
                    + "</td>"
                    + &format!("<td>{bolded}</td>")
                    + "</tr>"
            })
            .join("\n")
        + "\n</table>"
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
        ("badna", "maoja"),
        ("badri", "meo"),
        ("bajra", "jara"),
        ("bakni", "guobe"),
        ("bakri", "ganı"),
        ("balvi", "bıe"),
        ("bancu", "cuao"),
        ("bandu", "leoq"),
        ("banfi", "gumıe"),
        ("bangu", "zu"),
        ("banli", "suoı"),
        ("banro", "jeaq"),
        ("banzu", "bıaq"),
        // ("bapli", "caıtua"),
        ("barda", "sao"),
        ("bargu", "nuam"),
        ("bartu", "buı"),
        ("basti", "dıba"),
        ("batci", "kaqga"),
        ("batke", "cıoq"),
        ("bavmi", "'oshe"),
        ("bebna", "buoq"),
        ("benji", "dıeq"),
        ("berti", "bero"),
        ("besna", "kera"),
        ("betri", "zuom"),
        ("bevri", "hıe"),
        ("bi", "roaı"),
        ("bidju", "nupı"),
        ("bifce", "'apı"),
        ("bilma", "bıa"),
        ("binxo", "sho"),
        ("birka", "gıe"),
        ("bisli", "kıeı"),
        ("bitmu", "goeq"),
        ("blabi", "bao"),
        ("blanu", "mıo"),
        ("bliku", "gam"),
        ("bloti", "meaq"),
        ("bolci", "kıoq"),
        ("bongu", "kuoq"),
        ("botpi", "cheoq"),
        ("boxfo", "boe"),
        ("boxna", "sueq"),
        ("bradi", "ream"),
        ("bredi", "buo"),
        ("bridi", "jabı"),
        // ꝠAJUI BÁQ ZUDIUTOA MEOZUNO
        ("brife", "'ırue"),
        ("bu", "laı"),
        ("bukpu", "gueq"),
        ("bumru", "ceha"),
        ("bunre", "tıaq"),
        ("burcu", "chuım"),
        ("burna", "'eıla"),
        ("cabna", "naı"),
        ("cabra", "kea"),
        ("cacra", "hora"),
        ("cadzu", "koı"),
        ("cafne", "faı"),
        ("cakla", "choko"),
        ("calku", "pıu"),
        ("canci", "shıao"),
        ("cando", "suo"),
        // suo: on but doing nothing
        // dom: off
        ("canko", "chuao"),
        ("canlu", "goa"),
        ("carmi", "caı"),
        ("carna", "muoı"),
        ("carvi", "ruq"),
        ("casnu", "keoı"),
        ("catke", "dem"),
        ("catlu", "kaqsı"),
        ("catni", "cue"),
        ("catra", "jıam"),
        ("cecla", "cara"),
        ("cenba", "beo"),
        ("centi", "ceqtı"),
        ("cerni", "hoeı"),
        ("certu", "joe"),
        ("cevni", "jıao"),
        ("cfari", "ceo"),
        ("cfika", "lua"),
        ("cfila", "tuoı"),
        ("cfine", "lıeq"),
        ("ci", "saq"),
        ("ciblu", "sıaı"),
        ("cicna", "kuao"),
        ("cidja", "haq"),
        ("cidni", "bea"),
        ("cidro", "hıdo"),
        ("cifnu", "bem"),
        ("cigla", "guele"),
        ("cikna", "shıe"),
        ("cilce", "puaı"),
        ("cilmo", "cuaı"),
        ("cilre", "chıe"),
        ("cimni", "mıq"),
        ("cindu", "heıga"),
        ("cinfo", "labı"),
        ("cinki", "chom"),
        ("cinla", "ruoı"),
        ("cinmo", "moe"),
        ("cinri", "sıgı"),
        ("cinse", "seje"),
        ("cipni", "shuao"),
        ("cipra", "mıeq"),
        ("ciska", "kaı"),
        ("cisma", "nhame"),
        ("ciste", "doem"),
        ("cirla", "kuha"),
        ("citka", "chuq"),
        ("citno", "nıo"),
        // ("citri", "pudıu"),
        ("cizra", "nasa"),
        ("ckabu", "gola"),
        ("ckafi", "kafe"),
        ("ckaji", "'ıq"),
        ("ckape", "hıam"),
        ("ckiku", "'echı"),
        ("ckini", "cuoı"),
        ("ckire", "kıe"),
        ("ckule", "chıejıo"),
        ("ckunu", "'ukomuao"),
        ("clani", "buaı"),
        ("claxu", "cıa"),
        ("clira", "chuı"),
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
        ("co'e", "hao"),
        ("condi", "shoa"),
        ("cortu", "noı"),
        ("cpacu", "nua"),
        ("cpana", "neo"),
        ("cpedu", "sue"),
        ("crane", "shaq"),
        ("creka", "shatı"),
        ("cribe", "hako"),
        ("crida", "'aıpı"),
        ("crino", "rıq"),
        ("cripu", "coa"),
        ("ctino", "boaq"),
        ("ctuca", "gale"),
        ("cukla", "feoq"),
        ("cukta", "kue"),
        ("cumki", "daı"),
        ("cumla", "nıuq"),
        ("cunso", "neq"),
        ("cuntu", "tue"),
        ("cupra", "shuaq"),
        ("curmi", "shoe"),
        ("curnu", "nuq"),
        ("cusku", "kuq"),
        ("cutci", "puefuq"),
        ("cutne", "toraq"),
        ("cuxna", "koe"),
        ("da", "raı"),
        ("dacti", "raı"),
        ("dakfu", "torea"),
        ("dakli", "cea"),
        ("damba", "soı"),
        ("dandu", "beaı"),
        ("danlu", "nıaı"),
        ("danmo", "goq"),
        ("dansu", "marao"),
        ("daplu", "'aomo"),
        ("dargu", "tıeq"),
        ("darno", "jao"),
        ("darxi", "dea"),
        ("dasni", "geı"),
        ("daspo", "haıda"),
        ("dasri", "gıa"),
        ("datka", "heqtı"),
        ("datni", "dao"),
        ("degji", "cheı"),
        ("decti", "desı"),
        ("dekto", "heı"),
        ("dembi", "faseo"),
        ("denci", "nıoq"),
        ("denmi", "juıtaq"),
        ("denpa", "lao"),
        ("dertu", "'asaı"),
        ("desku", "furı"),
        ("detri", "daqchıu"),
        ("dilnu", "puao"),
        ("dikca", "ceoq"),
        ("dikni", "dıaq"),
        ("dinju", "jıo"),
        ("dirce", "zıa"),
        ("dirgo", "dıao"),
        ("dizlo", "nıa"),
        ("djacu", "nao"),
        ("djedi", "chaq"),
        ("djica", "shao"),
        ("djine", "feoq"),
        ("djuno", "dua"),
        ("do", "suq"),
        ("donri", "dıo"),
        ("drani", "due"),
        ("drata", "heo"),
        ("du", "jeq"),
        ("dukse", "duı"),
        ("dukti", "gıq"),
        ("dunda", "do"),
        ("dunli", "jeq"),
        ("facki", "gaı"), // ???
        ("fadni", "cem"),
        ("fagri", "loe"),
        ("fancu", "tıem"),
        ("fanmo", "fao"),
        ("fanta", "boq"), // maybe zua
        ("farlu", "shua"),
        ("farna", "feo"),
        ("farvi", "beo"),
        ("fasnu", "faq"),
        ("fatne", "nuq'o"),
        ("femti", "femto"),
        ("fengu", "feı"),
        ("fenki", "cheba"),
        ("festi", "mute"),
        ("fetsi", "lıq"),
        ("finpe", "cıe"),
        ("finti", "fıeq"),
        ("flalu", "juao"),
        ("flani", "cefa"),
        ("flecu", "hıu"),
        ("fliba", "buaq"),
        ("flira", "shom"),
        ("foldi", "dueq"),
        ("fonxa", "foq"),
        ("frati", "cua"),
        ("fraxu", "ruao"),
        ("frica", "heo"),
        ("friko", "'afarı"),
        ("frili", "fuı"),
        ("fukpi", "kopı"),
        ("funca", "neq"),
        ("fuzme", "caq'eı"),
        ("gacri", "tıe"),
        ("galfi", "beo"),
        ("galtu", "gea"),
        ("ganlo", "poa"),
        ("ganra", "nea"),
        ("ganse", "gaı"),
        ("ganzu", "suım"),
        ("gapci", "shına"),
        ("gapru", "gao"),
        ("gasnu", "tua"),
        ("genja", "'aka"),
        ("genxu", "gıuq"),
        ("gerku", "kune"),
        ("gerna", "zujuao"),
        ("gigdo", "gıga"),
        ("girzu", "me"),
        ("glare", "loq"),
        ("gleki", "jaı"),
        ("gletu", "seaq"),
        ("glico", "'ıqlı"),
        ("gocti", "ꝡoco"),
        ("gotro", "ꝡota"),
        ("grafu", "ceme"),
        ("grake", "garam"),
        ("grana", "beaq"),
        ("grasu", "nulı"),
        ("greku", "fuom"),
        ("grusi", "ruı"),
        ("grute", "zeo"),
        ("gubni", "cueq"),
        ("gugde", "gua"),
        ("gunka", "guaı"),
        ("gunma", "me"),
        ("gunro", "geoı"),
        ("gunta", "seraq"),
        ("gurni", "guı"),
        ("gusni", "gıo"),
        ("inda", "deaı"),
        ("ja", "ra"),
        ("jadni", ""),
        ("jalge", "se"),
        ("jamfu", "pue"),
        ("jamna", "soı"),
        ("janco", "shıo"),
        ("janli", "chuoq"),
        ("jarco", "'ıjo"),
        ("jarki", "zuı"),
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
        ("jeftu", "joa"),
        ("jei", "ma"),
        ("jelca", "hoaq"),
        ("jemna", "lıem"),
        ("jenca", "gıeq"),
        ("jetnu", "juna"),
        ("jgalu", "ceaq"),
        ("jgari", "jıaı"),
        ("jgina", "genea"),
        ("jgira", "hıoq"),
        ("jgita", "gıta"),
        ("jguna", "fıuq"),
        ("jibni", "juı"),
        ("jibri", "che"), // ?
        ("jicla", "jueq"),
        ("jicmu", "beoq"),
        ("jikca", "soao"),
        ("jikru", "seaı"),
        ("jimca", "gıaı"),
        ("jimpe", "lım"),
        ("jinme", "loha"),
        ("jinru", "shoem"),
        ("jinsa", "sıq"),
        ("jinvi", "mıu"),
        ("jipci", "goso"),
        ("jipno", "jıeq"),
        ("jirna", "tuaq"),
        ("jitfa", "sahu"),
        ("jitro", "caq"),
        ("jivna", "soqluaq"),
        ("jmaji", "kueq"),
        ("jmive", "mıe"),
        ("joi", "roı"),
        ("jorne", "coe"),
        ("judri", "chıu"),
        ("jufra", "kune"),
        ("jukpa", "haqbaı"),
        ("julne", "koaı"),
        ("jundi", "sı"),
        ("junri", "juaı"),
        ("junla", "jam"),
        ("jurme", "geaı"),
        ("kabri", "bıo"),
        // ("kagni", ""), // fıuq?
        ("kakne", "deq"),
        ("kajde", "zaru"),
        // ("kakpa", "huaı"),
        ("kalci", "keq"),
        ("kalri", "rıa"),
        ("kamni", "fuaı"),
        ("kanji", "jıoq"),
        ("kanla", "kaq"),
        ("kanro", "roe"),
        ("kansa", "gaq"),
        ("kantu", "toaı"),
        ("karce", "chao"),
        ("karda", "kata"),
        ("katna", "toe"),
        ("kecti", "koem"),
        ("kekti", "kueco"),
        ("kelci", "luaq"),
        ("kensa", "sheamı"),
        ("kerfa", "kıaq"),
        ("kerlo", "moma"),
        ("ketro", "kueta"),
        ("kevna", "jeoq"),
        ("kibro", "zıq"),
        ("kilto", "bıq"),
        // ("kijno", ""), // toaq why do you have 7 words for oxygen
        ("kinli", "choı"),
        ("klaku", "roq"),
        ("klama", "fa"),
        ("klani", "nhe"),
        ("klesi", "rıoq"),
        ("kluza", "huao"),
        ("korbi", "rea"),
        ("krasi", "sıao"),
        ("krati", "coq"),
        ("krefu", "guo"),
        ("krici", "chı"),
        ("krinu", "kuıca"),
        ("krixa", "shoı"),
        ("kruca", "chıeq"),
        ("kruji", "rujı"),
        ("kubli", "gam"),
        ("kufra", "foaq"),
        ("kulnu", "cıao"),
        ("kumfa", "kua"),
        ("kunra", "pıo"),
        ("kunti", "shea"),
        ("kurji", "kıaı"),
        ("kurki", "shueq"),
        ("lacpu", "baga"),
        ("ladru", "noaı"),
        ("lakne", "le"),
        ("lakse", "paku"),
        ("laldo", "geo"),
        ("lalxu", "soraq"),
        ("lamji", "leaq"),
        ("lanme", "hobı"),
        ("lanka", "laqka"),
        ("lanzu", "luo"),
        ("larcu", "lea"),
        ("lebna", "nua"),
        ("lelxe", "rere"),
        ("lerci", "reoq"),
        ("lerfu", "laı"),
        ("lifri", "lıe"),
        ("limna", "lıaı"),
        ("linji", "gıu"),
        ("linto", "lıu"),
        ("lisri", "lua"),
        ("liste", "mekao"),
        ("litki", "leu"),
        ("litru", "fa"),
        ("logji", "lojı"),
        ("lojbo", "lojıbaq"),
        ("loldi", "deaq"),
        ("lorxu", "hupı"),
        ("lunra", "mıao"),
        ("lujvo", "metoa"),
        ("lumci", "sıqja"),
        ("mabla", "huı"),
        ("makcu", "koaq"),
        ("makfa", "majı"),
        ("maksi", "mueı"),
        ("mamta", "mama"),
        ("mapku", "chea"),
        ("mapra", "mara"),
        ("mapti", "tıao"),
        ("marce", "chao"),
        ("marji", "saı"),
        ("marxa", "chueq"),
        ("masno", "meoq"),
        ("masti", "jue"),
        ("mavji", "'ota"),
        ("megdo", "mega"),
        ("mei", "lıaq"),
        ("melbi", "de"),
        ("menli", "moıchu"),
        ("mentu", "mınu"),
        ("merko", "'usona"),
        ("merli", "mıeq"),
        ("mi", "jı"),
        ("midju", "chu"),
        ("mikce", "goı"),
        ("mikri", "'umı"),
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
        ("mledi", "lıao"),
        ("moi", "ko"),
        ("mokca", "moa"),
        ("moklu", "buq"),
        ("morji", "moaq"),
        ("morna", "guoteı"),
        ("morsi", "muoq"),
        ("mrilu", "dıeq"),
        ("mu", "fe"),
        ("mudri", "muaosaı"),
        ("mulno", "muo"),
        ("munje", "jıaq"),
        ("mupli", "mua"),
        ("murse", "rıao"),
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
        ("nanvi", "nhano"),
        ("narge", "kası"),
        ("narju", "naraq"),
        ("nazbi", "shıma"),
        ("nelci", "cho"),
        ("nenri", "nıe"),
        ("nibli", "she"),
        ("nicte", "nuaq"),
        ("nimre", "kero"),
        ("ninmu", "lıq"),
        ("nitcu", "chıa"),
        ("no", "sıa"),
        ("no'e", "tuao"),
        ("nobli", "ruaı"),
        ("notci", "juo"),
        ("nukni", "cuı"),
        ("nunmu", "zeq"),
        ("pa", "shı"),
        ("pacna", "zaı"),
        ("pagbu", "paq"),
        ("pagre", "peo"),
        ("pajni", "jıe"),
        ("palci", "cheom"),
        ("panje", "soqja"),
        ("panzi", "fu"),
        ("patfu", "'aba"),
        ("patlu", "mazı"),
        ("pe'a", "'aı"),
        ("pelji", "peq"),
        ("pelxu", "lue"),
        ("pencu", "puaq"),
        ("pendo", "paı"),
        ("pensi", "moı"),
        ("perli", "pıso"),
        ("pesxu", "dashı"),
        ("petso", "peta"),
        ("pezli", "nıuboe"),
        ("picti", "pıko"),
        ("pilji", "reu"),
        ("pilka", "pıu"),
        ("pilno", "choq"),
        ("pimlu", "hueı"),
        ("pinca", "pı"),
        ("pinji", "peso"),
        ("pinta", "bore"),
        ("pinxe", "pıe"),
        ("pipno", "pıano"),
        ("pixra", "fuaq"),
        ("pleji", "teq"),
        ("plini", "pıanete"),
        ("plise", "shamu"),
        ("plipe", "loma"),
        ("pluja", "pıao"),
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
        ("prina", "pam"),
        ("pritu", "sıaq"),
        ("proga", "rom"),
        ("pruce", "case"),
        ("punji", "tıdo"),
        ("punli", "beq"),
        ("purci", "pu"),
        ("purdi", "soaq"),
        ("purmo", "puo"),
        ("ralju", "joq"),
        ("rarna", "roa"),
        ("randa", "boaı"),
        ("rango", "tuaı"),
        ("ranji", "duo"),
        ("rapli", "chıo"),
        ("ratni", "'atom"),
        ("re", "gu"),
        ("rebla", "huoı"),
        ("rectu", "nueq"),
        ("remna", "req"),
        ("renro", "hıeq"),
        ("renvi", "fea"),
        ("respa", "saoro"),
        ("ricfu", "mıa"),
        ("rinka", "ca"),
        ("rinsa", "hıo"),
        ("rirni", "pao"),
        ("rirxe", "hıu"),
        ("ritli", "rıtı"),
        ("rivbi", "rıeq"),
        ("ro", "tu"),
        ("roi", "chıo"),
        ("rokci", "pıo"),
        ("ronci", "roto"),
        ("ronro", "rona"),
        ("rotsu", "moaı"),
        ("rozgu", "barua"),
        ("ruble", "rue"),
        ("rufsu", "koq"),
        ("runta", "befe"), // ti.zoaq
        ("rutni", "baıse"),
        ("sakci", "supu"),
        ("sakli", "lusha"),
        ("sakta", "tıuq"),
        ("sampu", "shuaı"),
        ("sance", "laq"),
        ("sanga", "suaq"),
        ("sanji", "chıaq"),
        ("sanli", "sheaq"),
        ("sarcu", "sua"),
        ("sarlu", "suzı"),
        ("sarji", "rıaı"),
        ("saske", "dıu"),
        ("savru", "noq"),
        ("sazri", "caq"),
        ("sefsi", "taq"),
        ("sefta", "rem"),
        ("selci", "toaı"),
        ("selfu", "lueq"),
        ("senpi", "zoaı"),
        ("senta", "boepaq"),
        ("sepli", "poe"),
        ("sevzi", "taq"),
        ("sfasa", "zoeq"),
        ("sidbo", "sıo"),
        ("sidju", "soa"),
        ("sigja", "sıga"),
        ("sikta", "loeı"),
        ("silka", "sırı"),
        ("simlu", "du"),
        ("simsa", "sıu"),
        ("simxu", "cheo"),
        ("since", "'oguı"),
        ("sinma", "doı"),
        ("sinxa", "laı"),
        ("sipna", "nuo"),
        ("slasi", "ꝡachım"),
        ("smaji", "luq"),
        ("sriji", "ruoq"),
        ("sisku", "joaı"),
        ("sisti", "shaı"),
        ("skami", "rom"),
        ("skari", "reo"),
        ("skicu", "juoı"),
        ("skori", "nhoq"),
        ("slabu", "zem"),
        ("slaka", "raku"),
        ("slami", "'acı"),
        ("slari", "soe"),
        ("slilu", "furı"),
        ("sluni", "kepa"),
        ("smacu", "muse"),
        ("smani", "sımı"),
        ("smela", "puruq"),
        ("smuci", "sokum"),
        ("smuni", "mıu"),
        ("snada", "taı"),
        ("snanu", "namı"),
        ("snidu", "sekuq"),
        ("snime", "nıao"),
        ("snura", "lom"),
        ("so", "neı"),
        ("sobde", "soꝡa"),
        ("so'a", "tujuı"),
        ("so'e", "kaga"),
        ("so'i", "puı"), // teeeeechnically tıopuı
        ("so'o", "keı"),
        ("so'u", "sım"),
        ("solji", "'eloa"),
        ("solri", "hoe"),
        ("sonci", "soıche"),
        ("sorcu", "shuo"),
        ("sovda", "poaı"),
        ("spati", "nıu"),
        ("speni", "seo"),
        ("spisa", "hea"),
        ("spofu", "zueq"),
        ("spoja", "notuq"),
        ("spuda", "cua"),
        ("sraji", "sheaq"),
        ("sraku", "gıaı"),
        ("srana", "raq"),
        ("srasu", "poıba"),
        ("srera", "gom"),
        ("sruma", "cuo"),
        ("sruri", "rıe"),
        ("stace", "rıem"),
        ("stani", "tanı"),
        ("stapa", "pueq"),
        ("stasu", "tuze"),
        ("stedu", "joqhua"),
        ("stela", "tıoq"),
        ("stici", "hore"),
        ("stidi", "dıe"),
        ("stizu", "tuım"),
        ("stuna", "hochu"),
        ("stuzi", "rıaq"),
        ("su'ei", "cheo"),
        ("su'o", "sa"),
        ("sudga", "gıao"),
        ("suksa", "'eka"),
        ("sumji", "neu"),
        ("sumne", "shıq"),
        ("sumti", "'aqmı"),
        ("surla", "sea"),
        ("sutra", "suaı"),
        ("tabno", "kabo"),
        ("tadji", "chase"),
        ("tadni", "sıom"),
        ("tagji", "rueq"),
        ("tamca", "tama"),
        ("tanbo", "toq"),
        ("tance", "leq"),
        ("tanxe", "tıaı"),
        ("tarbi", "deto"),
        ("tarci", "nuım"),
        ("tarmi", "teı"),
        ("tarti", "ruo"),
        ("tatru", "teaq"),
        ("tavla", "keoı"),
        ("taxfu", "fuq"),
        ("tcadu", "doaq"),
        ("tcana", "ce"),
        ("tcati", "chaı"),
        ("tcica", "cheu"),
        ("tcidu", "noaq"),
        ("tcika", "daqmoa"),
        ("tcila", "foaı"),
        ("tcini", "tue"),
        ("tcita", "daocoe"),
        ("temci", "daq"),
        ("tende", "lam"),
        ("tenfa", "teu"),
        ("terdi", "gaja"),
        ("terpa", "tea"),
        ("terto", "tera"),
        ("ti", "nı"),
        ("tigni", "reum"),
        ("tilju", "tuoq"),
        ("tinbe", "lıeı"),
        ("tirna", "huo"),
        ("tirxu", "tıqra"),
        ("titla", "duao"),
        ("to'e", "gıq"),
        ("tonga", "laqtoaı"),
        ("tordu", "doq"),
        ("torka", "tetabo"),
        ("traji", "soq"),
        ("trene", "chue"),
        ("tricu", "muao"),
        ("trixe", "tıa"),
        ("troci", "leo"),
        ("tsali", "caı"),
        ("tsani", "seoq"),
        ("tsiju", "poaı"),
        ("tubnu", "bıu"),
        ("tugni", "mıujeq"),
        ("tumla", "dueq"),
        ("tunba", "pıa"),
        ("tunlo", "tuo"),
        ("tuple", "shıaq"),
        ("turni", "cue"),
        ("tutci", "chuo"),
        ("vacri", "rıo"),
        ("vajni", "suao"),
        ("valsi", "toa"),
        ("vanci", "seum"),
        ("vasru", "heq"),
        ("vasxu", "ceu"),
        ("vecnu", "teqdo"),
        ("verba", "deo"),
        ("vikmi", "cıu"),
        ("vimcu", "shata"),
        ("vindu", "bısa"),
        ("viska", "kaq"),
        ("vitke", "choaq"),
        ("vitno", "poam"),
        ("vo", "jo"),
        ("vofli", "lıaı"),
        ("voksa", "choalaq"),
        ("vorme", "kıao"),
        ("vreta", "reaq"),
        ("vreji", "kao"),
        ("xa", "cı"),
        ("xabju", "bua"),
        ("xadba", "gıem"),
        ("xadni", "tuaı"),
        ("xajmi", "luaı"),
        ("xaksu", "'achoq"),
        ("xalbo", "buoq"),
        ("xamgu", "gı"),
        ("xamsi", "naomı"),
        ("xance", "muq"),
        ("xanto", "'elu"),
        ("xanri", "'aobı"),
        ("xarci", "muıq"),
        ("xarju", "suhu"),
        ("xasli", "'aqshe"),
        ("xatra", "juo"),
        ("xatsi", "'ato"),
        ("xebni", "loı"),
        ("xecto", "fue"),
        ("xedja", "cheja"),
        ("xekri", "kuo"),
        ("xexso", "'esa"),
        ("xinmo", "puım"),
        ("xirma", "'eku"),
        ("xislu", "sıoq"),
        ("xlali", "huı"),
        ("xlura", "suoq"),
        ("xrani", "hıao"),
        ("xrula", "rua"),
        ("xruti", "rıu"),
        ("xukmi", "seao"),
        ("xunre", "kıa"),
        ("xutla", "goaq"),
        ("xusra", "ruaq"),
        ("zabna", "gı"),
        ("zajba", "seom"),
        ("zanru", "gımıu"),
        ("zarci", "dıem"),
        ("zasni", "zue"),
        ("zasti", "jıq"),
        ("zbabu", "sabaq"),
        ("zbasu", "baı"),
        ("zdani", "bue"),
        ("zdile", "luaı"),
        ("ze", "dıaı"),
        ("zekri", "jucıte"),
        ("zenba", "jeaq"),
        ("zepti", "seco"),
        ("zetro", "seta"),
        ("zgana", "sı"),
        ("zgike", "gıaq"),
        ("zifre", "sheı"),
        ("zirpu", "loa"),
        ("zmadu", "huaq"),
        ("zukte", "tao"),
        ("zumri", "marıkı"),
        ("zunle", "lıo"),
        ("zunti", "bebaq"),
        ("zutse", "tuı"),
        ("zvati", "tı"),
        ("zviki", "hakıq"),
    ])
});
