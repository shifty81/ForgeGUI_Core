# Root Hygiene Policy

Patch intake is higher authority than cleanup. The internal PCC scans and applies approved root `.patch` transports before running hygiene. Hygiene is non-fatal, manifest-aware, preserves patches, moves only known operational residue/source backup files, and leaves unknown root files in place with warnings.
