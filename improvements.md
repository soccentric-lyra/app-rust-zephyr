# Planned improvements: app-rust-zephyr

Gaps found in review and the enhancements planned to close them, highest priority first. Priority 1 is next, 3 is later.

## Gaps

| Area | Gap |
|---|---|
| Tests | No tests. Neither `cargo test` on the host nor `ztest` is used. |
| README | Its own "Related projects" list describes `app-rust-freertos` as "Rust on Embassy". It should say Rust on FreeRTOS. |
| Docs | No architecture page. The reason for the thin C layer (which drivers the bindings do not reach) is only visible in the code. |
| Upstream gap | `boards/dt-rust.yaml:61` has the only real `TODO` in the five repos: it does not read the write and erase properties from the devicetree. It is a single upstream-style line, but it is visible. |
| Duplication | `frame.rs` (228 lines) differs from the other two Rust copies. |
| Lint | `make lint` and `scripts/lint.sh` both exist. It is not obvious which one CI runs. |
| Validation | No emulated run. Zephyr's `native_sim` and QEMU/Renode targets are not used. |

## Enhancements

| Priority | Enhancement | Result |
|---|---|---|
| 1 | Host tests for the shared protocol crate, plus `ztest` on `native_sim` for the C glue | Tests that run in public CI |
| 1 | GitHub Actions workflow with badge | Visible green status |
| 2 | Architecture page: which parts are Rust and which are C, and why; devicetree overlay; hand-over to the bootloader | Explains the design choices |
| 2 | List the C layer's remaining responsibilities and the upstream issues that would remove each one | Honest and credible |
| 2 | Replace the `TODO` in `dt-rust.yaml` with a link to the tracking issue or fix it | No loose ends visible |
| 2 | `cargo audit` and `cargo deny`, and pin the Zephyr and Rust-module revisions in `Dockerfile.zephyr` | Reproducible and safer |
| 3 | Merge `make lint` and `scripts/lint.sh` into one documented entry point | Less confusion |
| 3 | Renode or QEMU run with boot-log check | Hardware-free validation |

