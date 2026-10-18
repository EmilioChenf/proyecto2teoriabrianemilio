# Reporte de validación y guía de entrega

Fecha: **18 de octubre de 2026**, zona America/Guatemala.

**Estado funcional: PASS.** Implementación completa en Rust, sin dependencias
externas. Las pruebas y comandos siguientes se ejecutaron localmente. No se hizo
push ni merge remoto. Se conserva el historial anterior.

## Requisitos y evidencia

| Requisito | Estado | Evidencia |
| --- | --- | --- |
| Leer CFG y validar errores | PASS | `parser.rs`; 25 pruebas de parser/tokenizador/argumentos |
| Anulables y eliminación de epsilon | PASS | `simplification/epsilon.rs`; omisiones, ciclos y más de 64 ocurrencias |
| Cierres y eliminación de unitarias | PASS | `simplification/unit.rs`; ciclos y herencia transitiva |
| Generadores y alcanzables | PASS | `simplification/useless.rs`; alcanzabilidad recalculada |
| Conversión automática a CNF | PASS | `cnf.rs`; aislamiento de terminales y binarización |
| Detección y validación CNF | PASS | `is_cnf`, `validate_cnf`; `1-cnf.txt` reconocido como CNF |
| Algoritmos generales | PASS | 1248 decisiones contrastadas con un oráculo independiente |
| CYK con programación dinámica | PASS | `cyk.rs`; tabla triangular y todas las divisiones |
| Respuesta SÍ/NO | PASS | Códigos 0 y 1; CLI comprobada como proceso real |
| Tiempo exclusivo de CYK | PASS | `Instant` dentro de `recognize`; excluye parser, conversión y árbol |
| Parse tree | PASS | `tree.rs`; pruebas verifican producciones y hojas |
| Tokenización de palabras y puntuación | PASS | Paréntesis pegados, mayor coincidencia y Unicode |
| Gramáticas aritméticas del profesor | PASS | Ambas pasan los 12 casos requeridos |
| Gramática inglesa del enunciado | PASS | `examples/english.txt`, sin modificar sus producciones |
| Modo de entrada textual | PASS | `cargo run`, sin argumentos, solicita una frase inglesa |
| Documentación de diseño y discusión | PASS | `README.md` y este reporte |
| Historial y autoría equilibrada | PASS | 12 commits nuevos, 6 por integrante; consulta reproducible abajo |
| PDF principal | REVISADO | Dos páginas leídas y renderizadas |
| PDF de ejemplo | LIMITACIÓN EXTERNA | Cinco páginas con streams dañados; renderizado blanco |

La arquitectura, simplificación, CNF, recurrencia de CYK, complejidad y árbol
están explicados en el README. La CNF admite el inicial a la derecha cuando no
genera epsilon, en concordancia con `1-cnf.txt`.

## Resultados de pruebas

Entorno ejecutado: Windows x86_64 MSVC, `rustc 1.97.0`, `cargo 1.97.0`.

| Suite | Aprobadas | Fallidas | Ignoradas |
| --- | ---: | ---: | ---: |
| `tests/parser_tokenizer.rs` | 25 | 0 | 0 |
| `tests/transformations.rs` | 28 | 0 | 0 |
| `tests/cyk_examples.rs` | 20 | 0 | 0 |
| `tests/cli.rs` | 11 | 0 | 0 |
| `tests/language_equivalence.rs` | 2 | 0 | 0 |
| **Total** | **86** | **0** | **0** |

No hay doctests ni tests unitarios internos adicionales: los 86 tests están en
estas cinco suites de integración. Cada test puede comprobar múltiples entradas.
El oráculo compara 8 × 63 + 24 × 31 = **1248** decisiones de pertenencia, además
de la igualdad de los conjuntos de palabras acotadas antes y después de CNF.

| Validación ejecutada | Resultado |
| --- | --- |
| `cargo fmt` | Correcto |
| `cargo fmt --check` | PASS |
| `cargo check` | PASS, sin advertencias |
| `cargo test` | PASS, 86/86 |
| `cargo clippy --all-targets --all-features -- -D warnings` | PASS, sin advertencias |
| `cargo build --release` | PASS |
| `git diff --check` | PASS |

También se ejecutaron las pruebas y Clippy con `--offline` correctamente. La
aplicación no descarga crates para compilar.

## Ejecuciones comprobadas fuera de Cargo test

Se ejecutaron **33 casos de consola**, comprobando los códigos de salida.
Los resultados esperados coincidieron en los 33. La tabla muestra la medición
observada en una ejecución del binario debug, no una promesa de rendimiento.

| Entrada | Esperado | Obtenido `1.txt` | Tiempo µs | Obtenido `1-cnf.txt` | Tiempo µs |
| --- | --- | --- | ---: | --- | ---: |
| `id` | SÍ | SÍ | 24.100 | SÍ | 21.500 |
| `id + id` | SÍ | SÍ | 99.300 | SÍ | 43.200 |
| `id * id` | SÍ | SÍ | 48.000 | SÍ | 25.400 |
| `id * id + id` | SÍ | SÍ | 91.300 | SÍ | 47.300 |
| `(id)` | SÍ | SÍ | 27.300 | SÍ | 23.300 |
| `(id + id)` | SÍ | SÍ | 256.000 | SÍ | 42.600 |
| `(id * id) + id` | SÍ | SÍ | 112.100 | SÍ | 124.400 |
| `+ id` | NO | NO | 24.500 | NO | 31.300 |
| `id +` | NO | NO | 19.100 | NO | 19.000 |
| `id * + id` | NO | NO | 50.200 | NO | 26.200 |
| `( id` | NO | NO | 23.200 | NO | 21.400 |
| `id id` | NO | NO | 25.200 | NO | 39.100 |

| Archivo / modalidad | Entrada | Esperado | Obtenido | Tiempo µs |
| --- | --- | --- | --- | ---: |
| Inglés | `she eats a cake with a fork` | SÍ | SÍ | 60.600 |
| Inglés | `the cat drinks the beer` | SÍ | SÍ | 81.900 |
| Inglés | `she a eats cake` | NO | NO | 73.900 |
| Inglés | `the cat the beer drinks` | NO | NO | 82.000 |
| Inglés, `--ignore-case` | `She eats a cake with a fork` | SÍ | SÍ | 115.900 |
| Epsilon | vacía | SÍ | SÍ | 13.100 |
| Epsilon | `a a a` | SÍ | SÍ | 48.500 |
| Inútiles | `c` | NO | NO | 4.900 |
| `1.txt` | `id @ id` | error léxico | código 2, carácter 4 | no se ejecuta CYK |

Para repetir las validaciones y estos mismos 33 casos en PowerShell:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\validar.ps1
```

La opción `-ExecutionPolicy Bypass` se aplica únicamente al proceso de validación;
no cambia políticas del sistema. Para repetir solo los ejemplos y construir debug:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\validar.ps1 -SoloEjemplos
```

El script falla si un comando o resultado no coincide y distingue NO (código 1)
de error (código 2). Restaura el PATH y la codificación al terminar.

## Historial y distribución

El punto de partida es `eef53cd` (commit inicial conservado). Los commits de este
desarrollo se pueden enumerar con sus hashes, autores, correos y mensajes reales:

```console
git log --reverse --format="%h | %an | %ae | %s" eef53cd..HEAD
git rev-list --count eef53cd..HEAD
git shortlog -sne eef53cd..HEAD
git shortlog -sne HEAD
git log --oneline --decorate -20
git status --short --branch
```

Distribución de los nuevos commits: **Emilio 6 / Brian 6**. Cada uno contiene
una unidad de trabajo real; no hay commits vacíos ni fechas alteradas.
Todos los mensajes nuevos están en español. La tabla de responsabilidades
registradas se encuentra en el README. La asistencia de Codex es explícita.

Se aplicó la identidad con `git -c user.name=... -c user.email=... commit`, sin
cambiar la configuración persistente de Git. El commit inicial mantiene el correo
anterior de Emilio; por eso el shortlog de todo HEAD lo lista por separado.

## Archivos de la entrega

- `Cargo.toml`, `Cargo.lock`: configuración y bloqueo, sin crates adicionales.
- `.gitignore`, `.gitattributes`: exclusión de `target/`, texto y PDF binario.
- `src/`: biblioteca modular y ejecutable, incluidos los tres submódulos de simplificación.
- `tests/`: cinco suites de integración.
- `examples/`: inglés, epsilon y símbolos inútiles.
- `1.txt`, `1-cnf.txt`: ejemplos originales, ahora registrados en Git.
- Los dos PDF originales, conservados y registrados en Git.
- `README.md`: guía, arquitectura, algoritmos, discusión y resultados.
- `docs/VALIDACION.md`: este reporte.
- `scripts/validar.ps1`: reproducción automática de checks y demostraciones.

No se incluyen archivos de herramientas PDF, renderizados temporales, binarios,
credenciales ni rutas absolutas del equipo en el código o documentación.

## Pendientes y límites

No quedan funciones incompletas ni fallos conocidos en los casos comprobados.
La única referencia que no pudo contrastarse es el PDF de ejemplo dañado:
`pypdf` informó errores de descompresión y PyMuPDF también; sus cinco páginas
renderizadas quedaron blancas. Se requiere otra copia para revisar ese material.
El PDF principal, los TXT y la solicitud sí se utilizaron.

El formato reserva `e`, `ε`, `|` y `->`; las palabras terminales usan minúsculas.
La tokenización usa máxima coincidencia, no explora todas las segmentaciones
léxicas posibles. El árbol corresponde a CNF y conserva auxiliares. Los límites
son 512 tokens por análisis y 100 000 producciones/combinaciones durante las
etapas de transformación. Son restricciones documentadas, con errores explícitos.

## Comandos para la demostración

Ejecutar desde la carpeta con `Cargo.toml`:

| Comando | Qué debe mostrar |
| --- | --- |
| `cargo run` | Solicita frase; escribir `she eats a cake with a fork` produce SÍ, tiempo y árbol |
| `cargo run -- examples/english.txt "the cat drinks the beer"` | SÍ, tiempo y árbol |
| `cargo run -- examples/english.txt "she a eats cake"` | NO; no existe árbol |
| `cargo run -- 1.txt "(id*id)+id"` | Original NO CNF; simplificación, CNF validada, SÍ y árbol |
| `cargo run -- 1-cnf.txt "id * id + id"` | Original SÍ CNF, aceptación y árbol |
| `cargo run -- 1.txt "id * + id"` | NO, código 1 |
| `cargo run -- --verbose 1.txt "id+id"` | Anulables X/Y, etapas intermedias y tabla |
| `cargo run -- --ignore-case examples/english.txt "She eats a cake with a fork"` | SÍ con entrada normalizada |
| `cargo run -- examples/epsilon.txt` | Presionar Enter: SÍ, árbol con ε |
| `cargo run -- 1.txt "id @ id"` | Error de tokenización, código 2 |
| `cargo test` | 86 tests aprobados |
| `cargo clippy --all-targets --all-features -- -D warnings` | Finaliza sin advertencias |

Los tiempos cambian por equipo, modo de compilación y carga de ejecución. Para
una medición optimizada puede usarse `cargo run --release -- 1.txt "id+id"`.
