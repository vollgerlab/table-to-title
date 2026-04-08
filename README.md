# table-to-title

**Live tool:** https://www.vollgerlab.com/table-to-title/

Paste or upload the biorxiv author submission table and get a formatted title-page author line with superscripted affiliations, equal-contribution (`*`) and corresponding-author (`†`) markers, a numbered affiliation list, and optional ORCiDs. Output can be downloaded as `.docx` or `.txt`.

## Usage

Go to https://www.vollgerlab.com/table-to-title/, paste your TSV or upload the file directly from biorxiv's author submission spreadsheet, and click **Download .docx**.

### Expected columns

The tool expects the standard biorxiv author table columns:

| Column | Notes |
|--------|-------|
| Email | |
| First Name | |
| Middle Name(s)/Initial(s) | |
| Last Name | |
| Suffix | |
| Institution | Separate multiple affiliations with `;` |
| Corresponding Author | Any non-empty value = yes |
| Home Page URL | Optional, ignored |
| Collaborative Group/Consortium | Optional, ignored |
| ORCiD | Optional |
| Equal Contribution | Any non-empty value = yes |

### Options

- **Show \* for equal contribution** — adds `*` superscript and legend line
- **Show † for corresponding authors** — adds `†` superscript and legend line
- **Include ORCiD list** — appends a linked ORCiD list below the affiliations

### Output format

```
Jane A. Doe1*, Bob T. Smith1,2*, Mitchell R. Vollger3†

1. Department of Genome Sciences, University of Washington School of Medicine, Seattle, WA, USA
2. Lewis-Sigler Institute for Integrative Genomics, Princeton University, Princeton, NJ, USA
3. Department of Human Genetics, University of Utah, Salt Lake City, UT, USA

* These authors contributed equally to this work.
†Corresponding author(s).
```

## Building locally

Requires [Rust](https://rustup.rs) and [wasm-pack](https://rustwasm.github.io/wasm-pack/).

```bash
# Install wasm-pack (if needed)
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build WASM and serve
wasm-pack build --target web --no-default-features --features wasm
cp -r pkg/* web/pkg/
python3 -m http.server 8080 --directory web/
open http://localhost:8080
```

The WASM build runs entirely in the browser — no server process required at runtime.

### Running tests

```bash
cargo test
```

## Deployment

Pushes to `main` automatically build and deploy to GitHub Pages via the workflow in `.github/workflows/deploy.yml`.
