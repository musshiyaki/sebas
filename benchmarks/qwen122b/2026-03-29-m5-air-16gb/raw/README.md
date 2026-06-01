# Raw Logs

Raw logs for the 2026-03-29 bring-up were not preserved in the public
repository. This directory exists to make that gap explicit.

For the next benchmark run, collect raw logs with:

```bash
tools/collect-qwen122b-repro-pack --case all
```

Then either:

- attach the generated archive to a GitHub Release, or
- promote small curated logs into a new tracked pack under `benchmarks/qwen122b/`.
