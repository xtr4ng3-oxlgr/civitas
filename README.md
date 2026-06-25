# CIVITAS

<img width="1672" height="941" alt="civitas" src="https://github.com/user-attachments/assets/86cb09df-7351-4dbe-bd07-1c51e3bfe25d" />


**CIVITAS** es una plataforma local de evidencia digital e inteligencia de estafas diseñada para organizar reportes ciudadanos, preservar evidencia, extraer entidades, construir líneas temporales, vincular indicadores y generar informes estructurados.

El proyecto utiliza un núcleo en **Rust** para la lógica principal y una interfaz visual en **Python** para operar el sistema desde una cabina local.

---

## Propósito

CIVITAS está pensado para ayudar a ordenar casos donde una persona necesita reunir evidencia digital de forma clara, verificable y presentable.

Casos posibles:

- estafas por transferencia;
- phishing;
- perfiles falsos;
- amenazas digitales;
- suplantación de identidad;
- ventas fraudulentas;
- enlaces sospechosos;
- conversaciones con datos relevantes;
- comprobantes, capturas, correos y archivos relacionados.

---

## Arquitectura

```text
Rust Core      motor principal, hashing, entidades, timeline, grafo, reportes
Python UI      interfaz visual con paneles, ASCII, controles y tablero local
C++ Helper     módulo opcional de metadatos nativos en Windows
PowerShell     extracción auxiliar de firma digital
JSON Store     base local portable
HTML/CSS       reportes profesionales
Batch          scripts de instalación, ejecución y build
```

---

## Funciones principales

- Crear casos.
- Registrar evidencia.
- Copiar evidencia al vault local.
- Calcular hashes SHA256 y SHA1.
- Extraer entidades desde archivos de texto.
- Detectar emails, URLs, teléfonos, CBU/CVU, CUIT/CUIL, alias, montos y wallets.
- Construir línea temporal.
- Construir grafo de vínculos entre entidades.
- Agregar notas del investigador.
- Registrar cadena de custodia.
- Generar reportes HTML y JSON.
- Exportar paquete ZIP del caso.
- Operar desde CLI o desde interfaz visual.

---

## Modelo de seguridad y uso permitido

CIVITAS es una herramienta de seguridad defensiva, documentación y organización de evidencia local.

La herramienta no está diseñada para:

- hackear cuentas;
- romper contraseñas;
- evadir permisos;
- espiar dispositivos;
- obtener ubicaciones privadas;
- consultar bases de datos filtradas;
- doxxear personas;
- acosar, perseguir o intimidar;
- automatizar vigilancia no autorizada;
- ejecutar acciones ofensivas contra terceros.

Uso previsto:

- organizar evidencia aportada voluntariamente;
- preservar archivos relacionados con un caso;
- calcular hashes;
- extraer entidades desde archivos propios;
- relacionar indicadores dentro del propio caso;
- generar reportes para soporte, denuncia, análisis interno o documentación;
- mantener trazabilidad del trabajo realizado.

---

## Aviso sobre modificaciones del código

El autor no avala modificaciones, bifurcaciones, redistribuciones o usos derivados que alteren el propósito defensivo, local y documental de CIVITAS.

Cualquier cambio orientado a vigilancia abusiva, intrusión, extracción no autorizada de datos, acoso, doxxing, evasión de permisos, explotación, automatización ofensiva o daño a terceros queda expresamente fuera del propósito del proyecto y no cuenta con aprobación del autor.

El código se entrega para fines defensivos, educativos, técnicos y de organización de evidencia. Quien modifique o redistribuya el proyecto asume la responsabilidad completa por sus cambios y por el uso posterior de esas versiones.

---

## Instalación

### 1. Compilar el núcleo Rust

```bat
build_windows\BUILD_CORE_RUST.bat
```

O manualmente:

```bash
cd core-rust
cargo build --release
```

El binario se copia a:

```text
bridge/civitas_core.exe
```

### 2. Instalar dependencias de la interfaz Python

```bat
INSTALAR_UI_PYTHON.bat
```

### 3. Abrir la interfaz visual

```bat
ABRIR_CIVITAS_UI.bat
```

---

## Uso por CLI

Crear caso:

```bash
bridge/civitas_core.exe case new "marketplace-fraud" --case-type "scam-report" --investigator "xtr4ng3" --description "Reporte ciudadano de posible estafa" --json
```

Agregar evidencia:

```bash
bridge/civitas_core.exe evidence add <case_id> "C:\Users\User\Downloads\chat.txt" --copy --tags "chat,whatsapp" --json
```

Extraer entidades:

```bash
bridge/civitas_core.exe entities extract <case_id> --json
```

Construir timeline:

```bash
bridge/civitas_core.exe timeline build <case_id> --json
```

Construir grafo:

```bash
bridge/civitas_core.exe graph build <case_id> --json
```

Generar reporte:

```bash
bridge/civitas_core.exe report html <case_id> --json
```

Exportar caso:

```bash
bridge/civitas_core.exe export <case_id>
```

---

## Workspace local

```text
civitas_workspace/
├─ civitas_store.json
├─ cases/
├─ reports/
├─ exports/
└─ vault/
```

---

## Limitaciones

CIVITAS no reemplaza una investigación judicial, pericia oficial, asesoramiento legal ni trabajo de una autoridad competente.

No confirma culpabilidad.  
No identifica personas por sí mismo.  
No garantiza que una entidad extraída sea maliciosa.  
No debe usarse para acusar sin verificación humana.

---

## # Licencia

<img width="300" height="159" alt="giphy (25)" src="https://github.com/user-attachments/assets/021720ff-3aec-4916-9a93-25d47afd7d97" />

**xtr4ng3**

MIT.
