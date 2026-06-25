use anyhow::{Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha1::{Digest as Sha1Digest, Sha1};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use uuid::Uuid;
use walkdir::WalkDir;
use zip::write::FileOptions;

const APP_NAME: &str = "CIVITAS";
const VERSION: &str = "1.0.0";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct CaseRecord { id: String, title: String, case_type: String, investigator: String, description: String, status: String, created_at: String, updated_at: String }

#[derive(Debug, Serialize, Deserialize, Clone)]
struct EvidenceRecord { id: String, case_id: String, source_path: String, vault_path: String, file_name: String, kind: String, size_bytes: u64, sha256: String, sha1: String, added_at: String, tags: Vec<String> }

#[derive(Debug, Serialize, Deserialize, Clone)]
struct EntityRecord { id: String, case_id: String, entity_type: String, value: String, source_evidence_id: String, observed_at: String }

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TimelineEvent { id: String, case_id: String, timestamp: String, source: String, title: String, detail: String, evidence_id: String }

#[derive(Debug, Serialize, Deserialize, Clone)]
struct LinkRecord { id: String, case_id: String, from_type: String, from_value: String, to_type: String, to_value: String, relation: String, evidence_id: String }

#[derive(Debug, Serialize, Deserialize, Clone)]
struct NoteRecord { id: String, case_id: String, actor: String, body: String, created_at: String }

#[derive(Debug, Serialize, Deserialize, Clone)]
struct CustodyEvent { id: String, case_id: String, evidence_id: String, actor: String, action: String, details: String, timestamp: String }

#[derive(Debug, Serialize, Deserialize, Default)]
struct Store { cases: Vec<CaseRecord>, evidence: Vec<EvidenceRecord>, entities: Vec<EntityRecord>, timeline: Vec<TimelineEvent>, links: Vec<LinkRecord>, notes: Vec<NoteRecord>, custody: Vec<CustodyEvent> }

#[derive(Parser)]
#[command(name = "civitas_core")]
#[command(version = VERSION)]
#[command(about = "CIVITAS local digital evidence and scam intelligence core")]
struct Cli { #[command(subcommand)] command: Commands }

#[derive(Subcommand)]
enum Commands {
    Init,
    Status { #[arg(long)] json: bool },
    Case { #[command(subcommand)] command: CaseCommand },
    Evidence { #[command(subcommand)] command: EvidenceCommand },
    Entities { #[command(subcommand)] command: EntityCommand },
    Timeline { #[command(subcommand)] command: TimelineCommand },
    Graph { #[command(subcommand)] command: GraphCommand },
    Note { #[command(subcommand)] command: NoteCommand },
    Report { #[command(subcommand)] command: ReportCommand },
    Export { case_id: String },
}

#[derive(Subcommand)]
enum CaseCommand {
    New { title: String, #[arg(long, default_value="scam-report")] case_type: String, #[arg(long, default_value="xtr4ng3")] investigator: String, #[arg(long, default_value="")] description: String, #[arg(long)] json: bool },
    List { #[arg(long)] json: bool },
    Show { case_id: String, #[arg(long)] json: bool },
}

#[derive(Subcommand)]
enum EvidenceCommand {
    Add { case_id: String, path: PathBuf, #[arg(long)] copy: bool, #[arg(long, default_value="")] tags: String, #[arg(long, default_value="xtr4ng3")] actor: String, #[arg(long)] json: bool },
    List { case_id: String, #[arg(long)] json: bool },
}

#[derive(Subcommand)]
enum EntityCommand { Extract { case_id: String, #[arg(long)] json: bool }, List { case_id: String, #[arg(long)] json: bool } }
#[derive(Subcommand)]
enum TimelineCommand { Build { case_id: String, #[arg(long)] json: bool }, List { case_id: String, #[arg(long)] json: bool } }
#[derive(Subcommand)]
enum GraphCommand { Build { case_id: String, #[arg(long)] json: bool }, List { case_id: String, #[arg(long)] json: bool } }
#[derive(Subcommand)]
enum NoteCommand { Add { case_id: String, body: String, #[arg(long, default_value="xtr4ng3")] actor: String, #[arg(long)] json: bool }, List { case_id: String, #[arg(long)] json: bool } }
#[derive(Subcommand)]
enum ReportCommand { Html { case_id: String, #[arg(long)] json: bool }, Json { case_id: String } }

fn main() -> Result<()> {
    let cli = Cli::parse();
    init_workspace()?;
    let mut store = load_store()?;

    match cli.command {
        Commands::Init => println!("{} workspace initialized", APP_NAME),
        Commands::Status { json } => print_payload(&serde_json::json!({"app": APP_NAME, "version": VERSION, "workspace": workspace().display().to_string(), "cases": store.cases.len(), "evidence": store.evidence.len(), "entities": store.entities.len(), "timeline_events": store.timeline.len(), "links": store.links.len()}), json)?,
        Commands::Case { command } => match command {
            CaseCommand::New { title, case_type, investigator, description, json } => {
                let case = CaseRecord { id: Uuid::new_v4().to_string(), title, case_type, investigator: investigator.clone(), description, status: "open".to_string(), created_at: now(), updated_at: now() };
                case_dir(&case.id)?;
                custody(&mut store, &case.id, "", &investigator, "case.created", "Case created");
                store.cases.push(case.clone());
                save_store(&store)?;
                print_payload(&case, json)?;
            }
            CaseCommand::List { json } => print_payload(&store.cases, json)?,
            CaseCommand::Show { case_id, json } => {
                get_case(&store, &case_id)?;
                print_payload(&serde_json::json!({"case": get_case(&store, &case_id)?, "evidence": evidence_for(&store, &case_id), "entities": entities_for(&store, &case_id), "timeline": timeline_for(&store, &case_id), "links": links_for(&store, &case_id), "notes": notes_for(&store, &case_id), "custody": custody_for(&store, &case_id)}), json)?;
            }
        },
        Commands::Evidence { command } => match command {
            EvidenceCommand::Add { case_id, path, copy, tags, actor, json } => {
                get_case(&store, &case_id)?;
                let added = add_evidence(&mut store, &case_id, &path, copy, &tags, &actor)?;
                save_store(&store)?;
                print_payload(&added, json)?;
            }
            EvidenceCommand::List { case_id, json } => { get_case(&store, &case_id)?; print_payload(&evidence_for(&store, &case_id), json)?; }
        },
        Commands::Entities { command } => match command {
            EntityCommand::Extract { case_id, json } => { get_case(&store, &case_id)?; let x = extract_entities(&mut store, &case_id)?; save_store(&store)?; print_payload(&x, json)?; }
            EntityCommand::List { case_id, json } => { get_case(&store, &case_id)?; print_payload(&entities_for(&store, &case_id), json)?; }
        },
        Commands::Timeline { command } => match command {
            TimelineCommand::Build { case_id, json } => { get_case(&store, &case_id)?; let x = build_timeline(&mut store, &case_id); save_store(&store)?; print_payload(&x, json)?; }
            TimelineCommand::List { case_id, json } => { get_case(&store, &case_id)?; print_payload(&timeline_for(&store, &case_id), json)?; }
        },
        Commands::Graph { command } => match command {
            GraphCommand::Build { case_id, json } => { get_case(&store, &case_id)?; let x = build_graph(&mut store, &case_id); save_store(&store)?; print_payload(&x, json)?; }
            GraphCommand::List { case_id, json } => { get_case(&store, &case_id)?; print_payload(&links_for(&store, &case_id), json)?; }
        },
        Commands::Note { command } => match command {
            NoteCommand::Add { case_id, body, actor, json } => {
                get_case(&store, &case_id)?;
                let note = NoteRecord { id: Uuid::new_v4().to_string(), case_id: case_id.clone(), actor: actor.clone(), body, created_at: now() };
                custody(&mut store, &case_id, "", &actor, "note.added", "Investigator note added");
                store.notes.push(note.clone());
                save_store(&store)?;
                print_payload(&note, json)?;
            }
            NoteCommand::List { case_id, json } => { get_case(&store, &case_id)?; print_payload(&notes_for(&store, &case_id), json)?; }
        },
        Commands::Report { command } => match command {
            ReportCommand::Html { case_id, json } => { get_case(&store, &case_id)?; let p = write_html_report(&store, &case_id)?; print_payload(&serde_json::json!({"report": p.display().to_string()}), json)?; }
            ReportCommand::Json { case_id } => { get_case(&store, &case_id)?; println!("{}", write_json_report(&store, &case_id)?.display()); }
        },
        Commands::Export { case_id } => { get_case(&store, &case_id)?; let h = write_html_report(&store, &case_id)?; let j = write_json_report(&store, &case_id)?; println!("{}", export_case(&store, &case_id, &[h, j])?.display()); }
    }
    Ok(())
}

fn workspace() -> PathBuf { std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join("civitas_workspace") }
fn store_path() -> PathBuf { workspace().join("civitas_store.json") }
fn init_workspace() -> Result<()> { fs::create_dir_all(workspace().join("cases"))?; fs::create_dir_all(workspace().join("reports"))?; fs::create_dir_all(workspace().join("exports"))?; fs::create_dir_all(workspace().join("vault"))?; if !store_path().exists() { save_store(&Store::default())?; } Ok(()) }
fn load_store() -> Result<Store> { if !store_path().exists() { return Ok(Store::default()); } Ok(serde_json::from_str(&fs::read_to_string(store_path())?).unwrap_or_default()) }
fn save_store(store: &Store) -> Result<()> { fs::write(store_path(), serde_json::to_string_pretty(store)?)?; Ok(()) }
fn case_dir(case_id: &str) -> Result<PathBuf> { let d = workspace().join("cases").join(case_id); fs::create_dir_all(d.join("evidence"))?; Ok(d) }
fn now() -> String { Utc::now().to_rfc3339() }
fn print_payload<T: Serialize>(payload: &T, _json_mode: bool) -> Result<()> { println!("{}", serde_json::to_string_pretty(payload)?); Ok(()) }
fn get_case<'a>(store: &'a Store, case_id: &str) -> Result<&'a CaseRecord> { store.cases.iter().find(|c| c.id == case_id).with_context(|| format!("case not found: {}", case_id)) }
fn evidence_for(store: &Store, case_id: &str) -> Vec<EvidenceRecord> { store.evidence.iter().filter(|x| x.case_id == case_id).cloned().collect() }
fn entities_for(store: &Store, case_id: &str) -> Vec<EntityRecord> { store.entities.iter().filter(|x| x.case_id == case_id).cloned().collect() }
fn timeline_for(store: &Store, case_id: &str) -> Vec<TimelineEvent> { let mut v: Vec<_> = store.timeline.iter().filter(|x| x.case_id == case_id).cloned().collect(); v.sort_by(|a,b| a.timestamp.cmp(&b.timestamp)); v }
fn links_for(store: &Store, case_id: &str) -> Vec<LinkRecord> { store.links.iter().filter(|x| x.case_id == case_id).cloned().collect() }
fn notes_for(store: &Store, case_id: &str) -> Vec<NoteRecord> { store.notes.iter().filter(|x| x.case_id == case_id).cloned().collect() }
fn custody_for(store: &Store, case_id: &str) -> Vec<CustodyEvent> { let mut v: Vec<_> = store.custody.iter().filter(|x| x.case_id == case_id).cloned().collect(); v.sort_by(|a,b| a.timestamp.cmp(&b.timestamp)); v }
fn custody(store: &mut Store, case_id: &str, evidence_id: &str, actor: &str, action: &str, details: &str) { store.custody.push(CustodyEvent { id: Uuid::new_v4().to_string(), case_id: case_id.to_string(), evidence_id: evidence_id.to_string(), actor: actor.to_string(), action: action.to_string(), details: details.to_string(), timestamp: now() }); }

fn hash_file(path: &Path) -> Result<(String, String)> { let mut f = File::open(path)?; let mut s256 = Sha256::new(); let mut s1 = Sha1::new(); let mut buf = [0u8; 1024 * 1024]; loop { let n = f.read(&mut buf)?; if n == 0 { break; } s256.update(&buf[..n]); s1.update(&buf[..n]); } Ok((hex::encode(s256.finalize()), hex::encode(s1.finalize()))) }
fn safe_name(v: &str) -> String { v.chars().map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' }).collect() }
fn kind(path: &Path) -> String { match path.extension().and_then(|x| x.to_str()).unwrap_or("").to_lowercase().as_str() { "txt"|"log"|"csv"|"json"|"md"|"html"|"xml" => "text", "png"|"jpg"|"jpeg"|"webp"|"bmp" => "image", "pdf" => "document", "zip"|"rar"|"7z" => "archive", "mp3"|"wav"|"ogg"|"m4a" => "audio", "mp4"|"mov"|"avi"|"mkv" => "video", _ => "file" }.to_string() }

fn add_evidence(store: &mut Store, case_id: &str, path: &Path, copy: bool, tags: &str, actor: &str) -> Result<Vec<EvidenceRecord>> {
    let path = path.canonicalize()?;
    let mut out = Vec::new();
    if path.is_file() { out.push(add_file(store, case_id, &path, copy, tags, actor)?); }
    if path.is_dir() { for e in WalkDir::new(&path).into_iter().filter_map(|x| x.ok()) { if e.path().is_file() { if let Ok(x) = add_file(store, case_id, e.path(), copy, tags, actor) { out.push(x); } } } }
    Ok(out)
}

fn add_file(store: &mut Store, case_id: &str, path: &Path, copy: bool, tags: &str, actor: &str) -> Result<EvidenceRecord> {
    let id = Uuid::new_v4().to_string();
    let meta = fs::metadata(path)?;
    let (sha256, sha1) = hash_file(path)?;
    let file_name = path.file_name().and_then(|x| x.to_str()).unwrap_or("evidence.bin").to_string();
    let mut vault_path = String::new();
    if copy { let target = case_dir(case_id)?.join("evidence").join(format!("{}_{}", id, safe_name(&file_name))); fs::copy(path, &target)?; vault_path = target.display().to_string(); }
    let rec = EvidenceRecord { id: id.clone(), case_id: case_id.to_string(), source_path: path.display().to_string(), vault_path, file_name, kind: kind(path), size_bytes: meta.len(), sha256, sha1, added_at: now(), tags: tags.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect() };
    store.evidence.push(rec.clone());
    custody(store, case_id, &rec.id, actor, "evidence.added", &format!("Evidence added: {}", rec.file_name));
    Ok(rec)
}

fn read_text(path: &Path) -> String { let ext = path.extension().and_then(|x| x.to_str()).unwrap_or("").to_lowercase(); if !["txt","log","csv","json","md","html","xml","ini","conf"].contains(&ext.as_str()) { return String::new(); } if let Ok(m) = fs::metadata(path) { if m.len() > 5_000_000 { return String::new(); } } fs::read_to_string(path).unwrap_or_default() }

fn extract_entities(store: &mut Store, case_id: &str) -> Result<Vec<EntityRecord>> {
    store.entities.retain(|x| x.case_id != case_id);
    let patterns: Vec<(&str, Regex)> = vec![
        ("email", Regex::new(r"(?i)\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b")?),
        ("url", Regex::new(r"(?i)\bhttps?://[^\s<>'\)]+")?),
        ("phone", Regex::new(r"(?x)(?:\+?\d{1,3}[\s.-]?)?(?:\(?\d{2,4}\)?[\s.-]?)?\d{3,4}[\s.-]?\d{3,4}")?),
        ("cbu_cvu", Regex::new(r"\b\d{22}\b")?),
        ("cuit_cuil", Regex::new(r"\b\d{2}-?\d{8}-?\d\b")?),
        ("alias", Regex::new(r"(?i)\b[a-z0-9]+(?:\.[a-z0-9]+){1,3}\b")?),
        ("amount", Regex::new(r"(?i)(?:\$|ARS|USD)\s?\d{1,3}(?:[.,]\d{3})*(?:[.,]\d{2})?")?),
        ("crypto_wallet", Regex::new(r"\b(?:bc1|[13])[a-zA-HJ-NP-Z0-9]{25,59}\b|\b0x[a-fA-F0-9]{40}\b")?),
    ];
    let mut seen = BTreeSet::new();
    let mut found = Vec::new();
    for ev in evidence_for(store, case_id) {
        let p = if !ev.vault_path.is_empty() { PathBuf::from(&ev.vault_path) } else { PathBuf::from(&ev.source_path) };
        let text = read_text(&p);
        for (etype, re) in &patterns {
            for m in re.find_iter(&text) {
                let value = m.as_str().trim().trim_matches(|c: char| c == ',' || c == '.' || c == ';').to_string();
                let key = format!("{}:{}:{}", case_id, etype, value.to_lowercase());
                if value.len() > 3 && seen.insert(key) {
                    let rec = EntityRecord { id: Uuid::new_v4().to_string(), case_id: case_id.to_string(), entity_type: etype.to_string(), value, source_evidence_id: ev.id.clone(), observed_at: now() };
                    store.entities.push(rec.clone());
                    found.push(rec);
                }
            }
        }
    }
    Ok(found)
}

fn build_timeline(store: &mut Store, case_id: &str) -> Vec<TimelineEvent> {
    store.timeline.retain(|x| x.case_id != case_id);
    let mut out = Vec::new();
    for ev in evidence_for(store, case_id) {
        out.push(TimelineEvent { id: Uuid::new_v4().to_string(), case_id: case_id.to_string(), timestamp: ev.added_at.clone(), source: "evidence".to_string(), title: "Evidence added".to_string(), detail: format!("{} added with SHA256 {}", ev.file_name, ev.sha256), evidence_id: ev.id.clone() });
    }
    for n in notes_for(store, case_id) {
        out.push(TimelineEvent { id: Uuid::new_v4().to_string(), case_id: case_id.to_string(), timestamp: n.created_at.clone(), source: "note".to_string(), title: "Investigator note".to_string(), detail: n.body, evidence_id: String::new() });
    }
    out.sort_by(|a,b| a.timestamp.cmp(&b.timestamp));
    store.timeline.extend(out.clone());
    out
}

fn build_graph(store: &mut Store, case_id: &str) -> Vec<LinkRecord> {
    store.links.retain(|x| x.case_id != case_id);
    let mut groups: BTreeMap<String, Vec<EntityRecord>> = BTreeMap::new();
    for e in entities_for(store, case_id) { groups.entry(e.source_evidence_id.clone()).or_default().push(e); }
    let mut out = Vec::new();
    for (evidence_id, group) in groups {
        for a in 0..group.len() { for b in (a+1)..group.len() {
            let l = LinkRecord { id: Uuid::new_v4().to_string(), case_id: case_id.to_string(), from_type: group[a].entity_type.clone(), from_value: group[a].value.clone(), to_type: group[b].entity_type.clone(), to_value: group[b].value.clone(), relation: "co_observed".to_string(), evidence_id: evidence_id.clone() };
            out.push(l.clone()); store.links.push(l);
        }}
    }
    out
}

fn esc(s: &str) -> String { s.replace('&',"&amp;").replace('<',"&lt;").replace('>',"&gt;").replace('"',"&quot;") }
fn rows<T, F: Fn(&T)->String>(v: Vec<T>, f: F) -> String { v.iter().map(f).collect::<Vec<_>>().join("\n") }

fn write_html_report(store: &Store, case_id: &str) -> Result<PathBuf> {
    let c = get_case(store, case_id)?;
    let evidence = rows(evidence_for(store, case_id), |e| format!("<tr><td>{}</td><td>{}</td><td>{}</td><td><code>{}</code></td><td><code>{}</code></td></tr>", esc(&e.file_name), esc(&e.kind), e.size_bytes, esc(&e.sha256), esc(&e.source_path)));
    let entities = rows(entities_for(store, case_id), |e| format!("<tr><td>{}</td><td><code>{}</code></td><td>{}</td></tr>", esc(&e.entity_type), esc(&e.value), esc(&e.source_evidence_id)));
    let timeline = rows(timeline_for(store, case_id), |t| format!("<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>", esc(&t.timestamp), esc(&t.source), esc(&t.title), esc(&t.detail)));
    let links = rows(links_for(store, case_id), |l| format!("<tr><td>{}: <code>{}</code></td><td>{}</td><td>{}: <code>{}</code></td></tr>", esc(&l.from_type), esc(&l.from_value), esc(&l.relation), esc(&l.to_type), esc(&l.to_value)));
    let custody_rows = rows(custody_for(store, case_id), |x| format!("<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>", esc(&x.timestamp), esc(&x.actor), esc(&x.action), esc(&x.details)));
    let notes = notes_for(store, case_id).iter().map(|n| format!("<div class='note'><b>{}</b> · {}<pre>{}</pre></div>", esc(&n.actor), esc(&n.created_at), esc(&n.body))).collect::<Vec<_>>().join("\n");
    let doc = format!(r#"<!doctype html><html lang="es"><head><meta charset="utf-8"><title>CIVITAS Report</title><style>body{{background:#05070b;color:#e8f6ff;font-family:Consolas,Segoe UI,Arial;padding:32px}}h1,h2{{color:#ff304f}}.card{{background:#0b1018;border:1px solid #1d2a3c;border-radius:16px;padding:18px;margin:18px 0}}table{{width:100%;border-collapse:collapse}}td,th{{border-bottom:1px solid #1d2a3c;padding:9px;text-align:left;vertical-align:top}}th{{color:#7ef9ff}}code{{color:#d6f7ff}}.muted{{color:#9fb1c7}}.note{{border-left:3px solid #ff304f;padding-left:12px;margin:12px 0}}</style></head><body>
<h1>CIVITAS</h1><p class="muted">Digital Evidence & Scam Intelligence Workbench · xtr4ng3</p>
<div class="card"><h2>Resumen del caso</h2><table><tr><th>ID</th><td><code>{}</code></td></tr><tr><th>Título</th><td>{}</td></tr><tr><th>Tipo</th><td>{}</td></tr><tr><th>Investigador</th><td>{}</td></tr><tr><th>Estado</th><td>{}</td></tr><tr><th>Descripción</th><td>{}</td></tr></table></div>
<div class="card"><h2>Evidencia preservada</h2><table><tr><th>Archivo</th><th>Tipo</th><th>Bytes</th><th>SHA256</th><th>Origen</th></tr>{}</table></div>
<div class="card"><h2>Entidades detectadas</h2><table><tr><th>Tipo</th><th>Valor</th><th>Evidencia</th></tr>{}</table></div>
<div class="card"><h2>Línea temporal</h2><table><tr><th>Fecha</th><th>Fuente</th><th>Título</th><th>Detalle</th></tr>{}</table></div>
<div class="card"><h2>Vínculos</h2><table><tr><th>Origen</th><th>Relación</th><th>Destino</th></tr>{}</table></div>
<div class="card"><h2>Notas</h2>{}</div>
<div class="card"><h2>Cadena de custodia</h2><table><tr><th>Fecha</th><th>Actor</th><th>Acción</th><th>Detalle</th></tr>{}</table></div>
</body></html>"#, esc(&c.id), esc(&c.title), esc(&c.case_type), esc(&c.investigator), esc(&c.status), esc(&c.description), evidence, entities, timeline, links, notes, custody_rows);
    let out = workspace().join("reports").join(format!("civitas_{}_report.html", case_id));
    fs::write(&out, doc)?;
    Ok(out)
}

fn write_json_report(store: &Store, case_id: &str) -> Result<PathBuf> {
    let payload = serde_json::json!({"case": get_case(store, case_id)?, "evidence": evidence_for(store, case_id), "entities": entities_for(store, case_id), "timeline": timeline_for(store, case_id), "links": links_for(store, case_id), "notes": notes_for(store, case_id), "custody": custody_for(store, case_id)});
    let out = workspace().join("reports").join(format!("civitas_{}_report.json", case_id));
    fs::write(&out, serde_json::to_string_pretty(&payload)?)?;
    Ok(out)
}

fn export_case(store: &Store, case_id: &str, extra: &[PathBuf]) -> Result<PathBuf> {
    let out = workspace().join("exports").join(format!("civitas_{}_case.zip", case_id));
    if out.exists() { fs::remove_file(&out)?; }
    let file = File::create(&out)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let root = case_dir(case_id)?;
    if root.exists() { for e in WalkDir::new(&root).into_iter().filter_map(|x| x.ok()) { if e.path().is_file() { let rel = e.path().strip_prefix(&root)?; zip.start_file(format!("case/{}/{}", case_id, rel.display()).replace('\\', "/"), options)?; let mut f = File::open(e.path())?; let mut b = Vec::new(); f.read_to_end(&mut b)?; zip.write_all(&b)?; } } }
    for fpath in extra { if fpath.exists() { let name = fpath.file_name().and_then(|x| x.to_str()).unwrap_or("report"); zip.start_file(format!("reports/{}", name), options)?; let mut f = File::open(fpath)?; let mut b = Vec::new(); f.read_to_end(&mut b)?; zip.write_all(&b)?; } }
    zip.start_file("manifest.json", options)?;
    zip.write_all(serde_json::to_string_pretty(&serde_json::json!({"app": APP_NAME, "version": VERSION, "case_id": case_id, "exported_at": now(), "evidence_count": evidence_for(store, case_id).len()}))?.as_bytes())?;
    zip.finish()?;
    Ok(out)
}
