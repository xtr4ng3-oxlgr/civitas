from __future__ import annotations

import json
import subprocess
from pathlib import Path
from typing import Any

try:
    from textual.app import App, ComposeResult
    from textual.containers import Horizontal, Vertical
    from textual.widgets import Header, Footer, Static, DataTable, Button, Label
except Exception as exc:
    print("CIVITAS UI requiere dependencias.")
    print("Ejecutá INSTALAR_UI_PYTHON.bat")
    print(f"Detalle: {exc}")
    raise SystemExit(1)

ROOT = Path(__file__).resolve().parents[1]
CORE = ROOT / "bridge" / "civitas_core.exe"

BANNER = r"""
 ██████╗██╗██╗   ██╗██╗████████╗ █████╗ ███████╗
██╔════╝██║██║   ██║██║╚══██╔══╝██╔══██╗██╔════╝
██║     ██║██║   ██║██║   ██║   ███████║███████╗
██║     ██║╚██╗ ██╔╝██║   ██║   ██╔══██║╚════██║
╚██████╗██║ ╚████╔╝ ██║   ██║   ██║  ██║███████║
 ╚═════╝╚═╝  ╚═══╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝╚══════╝

 DIGITAL EVIDENCE & SCAM INTELLIGENCE WORKBENCH
"""

class CoreBridge:
    def run(self, *args: str) -> Any:
        if not CORE.exists():
            raise FileNotFoundError("No se encontró bridge\\civitas_core.exe. Ejecutá build_windows\\BUILD_CORE_RUST.bat")
        command = [str(CORE), *args]
        if "--json" not in command:
            command.append("--json")
        proc = subprocess.run(command, cwd=str(ROOT), text=True, capture_output=True)
        if proc.returncode != 0:
            raise RuntimeError(proc.stderr.strip() or proc.stdout.strip() or "Core error")
        text = proc.stdout.strip()
        return json.loads(text) if text else {}

class CivitasApp(App):
    CSS = """
    Screen { background: #05070b; color: #e8f6ff; }
    #banner { height: 10; color: #ff304f; border: heavy #ff304f; padding: 1; }
    .box { border: heavy #1d2a3c; padding: 1; background: #0b1018; }
    #cases { width: 35%; }
    #workspace { width: 65%; }
    #log { height: 5; border: solid #1d2a3c; background: #060a10; }
    DataTable { height: 1fr; }
    Button { margin: 1; }
    """
    BINDINGS = [("q", "quit", "Salir"), ("r", "refresh", "Actualizar"), ("n", "new_case", "Nuevo"), ("e", "extract", "Entidades"), ("t", "timeline", "Timeline"), ("g", "graph", "Grafo"), ("h", "report", "Reporte")]

    def __init__(self) -> None:
        super().__init__()
        self.bridge = CoreBridge()
        self.active_case_id: str | None = None

    def compose(self) -> ComposeResult:
        yield Header(show_clock=True)
        yield Static(BANNER, id="banner")
        with Horizontal():
            with Vertical(id="cases", classes="box"):
                yield Label("CASES")
                yield DataTable(id="case_table")
                yield Button("Nuevo caso", id="new_case")
                yield Button("Actualizar", id="refresh")
            with Vertical(id="workspace", classes="box"):
                yield Label("ACTIVE CASE")
                yield Static("Sin caso seleccionado.", id="active_case")
                yield DataTable(id="detail_table")
                with Horizontal():
                    yield Button("Extraer entidades", id="extract")
                    yield Button("Timeline", id="timeline")
                    yield Button("Grafo", id="graph")
                    yield Button("Reporte", id="report")
        yield Static("CIVITAS UI lista.", id="log")
        yield Footer()

    def on_mount(self) -> None:
        self.query_one("#case_table", DataTable).add_columns("ID", "Título", "Tipo", "Estado")
        self.query_one("#detail_table", DataTable).add_columns("Módulo", "Valor", "Detalle")
        self.refresh_cases()

    def set_log(self, msg: str) -> None:
        self.query_one("#log", Static).update(msg)

    def refresh_cases(self) -> None:
        try:
            cases = self.bridge.run("case", "list")
            table = self.query_one("#case_table", DataTable)
            table.clear()
            for item in cases:
                table.add_row(item.get("id", "")[:8], item.get("title", ""), item.get("case_type", ""), item.get("status", ""), key=item.get("id", ""))
            self.set_log(f"Casos cargados: {len(cases)}")
        except Exception as exc:
            self.set_log(f"Error: {exc}")

    def on_data_table_row_selected(self, event: DataTable.RowSelected) -> None:
        if event.data_table.id == "case_table":
            self.active_case_id = str(event.row_key.value)
            self.load_case(self.active_case_id)

    def load_case(self, case_id: str) -> None:
        try:
            data = self.bridge.run("case", "show", case_id)
            case = data.get("case", {})
            self.query_one("#active_case", Static).update(f"{case.get('title')}\\nID: {case.get('id')}\\nTipo: {case.get('case_type')}\\nInvestigador: {case.get('investigator')}")
            detail = self.query_one("#detail_table", DataTable)
            detail.clear()
            detail.add_row("Evidencia", str(len(data.get("evidence", []))), "Archivos registrados")
            detail.add_row("Entidades", str(len(data.get("entities", []))), "Datos extraídos")
            detail.add_row("Timeline", str(len(data.get("timeline", []))), "Eventos")
            detail.add_row("Vínculos", str(len(data.get("links", []))), "Relaciones")
            detail.add_row("Notas", str(len(data.get("notes", []))), "Observaciones")
            self.set_log("Caso cargado.")
        except Exception as exc:
            self.set_log(f"Error: {exc}")

    def require_case(self) -> str | None:
        if not self.active_case_id:
            self.set_log("Seleccioná o creá un caso primero.")
            return None
        return self.active_case_id

    def action_refresh(self) -> None: self.refresh_cases()
    def action_new_case(self) -> None: self.create_case()
    def action_extract(self) -> None: self.run_case_action("entities", "extract", "Entidades extraídas")
    def action_timeline(self) -> None: self.run_case_action("timeline", "build", "Timeline construido")
    def action_graph(self) -> None: self.run_case_action("graph", "build", "Grafo construido")
    def action_report(self) -> None: self.run_case_action("report", "html", "Reporte generado")

    def create_case(self) -> None:
        try:
            data = self.bridge.run("case", "new", "nuevo-caso-civitas", "--case-type", "citizen-report", "--investigator", "xtr4ng3", "--description", "Caso creado desde CIVITAS UI.")
            self.active_case_id = data.get("id")
            self.refresh_cases()
            self.load_case(self.active_case_id)
        except Exception as exc:
            self.set_log(f"Error: {exc}")

    def run_case_action(self, module: str, action: str, label: str) -> None:
        case_id = self.require_case()
        if not case_id:
            return
        try:
            data = self.bridge.run(module, action, case_id)
            self.load_case(case_id)
            if isinstance(data, list):
                self.set_log(f"{label}: {len(data)}")
            elif isinstance(data, dict):
                self.set_log(f"{label}: {data}")
            else:
                self.set_log(label)
        except Exception as exc:
            self.set_log(f"Error: {exc}")

    def on_button_pressed(self, event: Button.Pressed) -> None:
        actions = {"new_case": self.action_new_case, "refresh": self.action_refresh, "extract": self.action_extract, "timeline": self.action_timeline, "graph": self.action_graph, "report": self.action_report}
        if event.button.id in actions:
            actions[event.button.id]()

if __name__ == "__main__":
    CivitasApp().run()
