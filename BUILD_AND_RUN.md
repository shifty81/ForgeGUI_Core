# Build and Run

Use `PROJECT_CONTROL_CENTER.cmd`. Full Gate runs metadata, fmt check, check, tests, strict Clippy, and a release Lab build.

If an overlaid/existing `Cargo.lock` is stale, Full Gate performs one controlled lock refresh only for Cargo's explicit stale-lock diagnostic, then returns to locked certification.
