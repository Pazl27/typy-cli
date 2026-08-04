#!/usr/bin/env python3
"""Generate resources/code/snippets.json for typy's coding mode.

Extracts short, comment-free functions from a curated set of GitHub repos,
mirroring speedtyper.dev's approach (whole-function nodes, size filtered).

Usage (inside the dev shell, which pins Python):
    nix develop -c python3 scripts/generate-snippets.py > resources/code/snippets.json

Set GITHUB_TOKEN for a higher GitHub API rate limit. Uses only the stdlib.
"""

import json, os, re, random, sys, urllib.request

TOKEN = os.environ.get("GITHUB_TOKEN", "")
random.seed(7)

REPOS = {
    "rust": ["rust-lang/cargo"],
    "python": ["pallets/flask", "tiangolo/fastapi"],
    "go": ["etcd-io/etcd"],
    "javascript": ["lodash/lodash"],
    "c": ["ggerganov/llama.cpp"],
    "cpp": ["ggerganov/llama.cpp"],
}
EXT = {"rust": ".rs", "python": ".py", "go": ".go", "javascript": ".js", "c": ".c", "cpp": ".cpp"}
EXCLUDE = ["test", "docs", "example", "migration", ".github", "benchmark", "vendor",
           "third_party", "3rdparty", "/build/", "generated", ".min.", "fuzz",
           ".pb.", "_pb2", ".gen.", ".g.dart", "mock", "snapshot"]

FILES_PER_REPO = 40
PER_LANG_CAP = 45
MIN_CHARS, MAX_CHARS = 100, 300
MAX_LINES = 11
MAX_LINE_LEN = 55

def api(url):
    req = urllib.request.Request(url, headers={"User-Agent": "typy-gen",
        "Accept": "application/vnd.github+json",
        **({"Authorization": f"token {TOKEN}"} if TOKEN else {})})
    with urllib.request.urlopen(req, timeout=30) as r:
        return json.load(r)

def raw(repo, branch, path):
    url = f"https://raw.githubusercontent.com/{repo}/{branch}/{path}"
    req = urllib.request.Request(url, headers={"User-Agent": "typy-gen"})
    with urllib.request.urlopen(req, timeout=30) as r:
        return r.read().decode("utf-8", "replace")

def cut_inline(line, marker):
    q = None; i = 0
    while i < len(line):
        c = line[i]
        if q:
            if c == "\\": i += 2; continue
            if c == q: q = None
        else:
            if c in "\"'": q = c
            elif line[i:i+len(marker)] == marker: return line[:i]
        i += 1
    return line

def strip_comments(code, lang):
    marker = "#" if lang == "python" else "//"
    out = []; in_block = False; block_end = ""
    for line in code.split("\n"):
        st = line.strip()
        if in_block:
            if block_end in line: in_block = False
            continue
        if lang == "python" and (st.startswith('"""') or st.startswith("'''")):
            d = st[:3]
            if not (len(st) > 3 and d in st[3:]): in_block = True; block_end = d
            continue
        if st.startswith("/*"):
            if "*/" not in st[2:]: in_block = True; block_end = "*/"
            continue
        cp = cut_inline(line, marker).rstrip()
        if cp.strip() == "":
            continue
        out.append(cp)
    return "\n".join(out)

def normalize(code):
    code = code.replace("\t", "    ")
    lines = [l.rstrip() for l in code.split("\n")]
    while lines and not lines[0].strip(): lines.pop(0)
    while lines and not lines[-1].strip(): lines.pop()
    return "\n".join(lines)

def indent(line): return len(line) - len(line.lstrip(" "))

def sig(line, lang):
    s = line.strip()
    if lang == "rust": return re.match(r"(pub(\([^)]*\))?\s+)?(default\s+)?(async\s+)?(unsafe\s+)?fn\s+\w", s)
    if lang == "go": return re.match(r"func\s+(\([^)]*\)\s+)?\w", s)
    if lang == "javascript": return re.match(r"(export\s+)?(default\s+)?(async\s+)?function\s+\w", s)
    if lang in ("c", "cpp"):
        if s.startswith(("if", "for", "while", "switch", "return", "else", "#", "//", "template", "typedef")): return False
        return re.match(r"[A-Za-z_][\w:<>,*&\s]*[ \t*&]+[A-Za-z_]\w*\s*\([^;{]*\)\s*(const)?\s*\{?\s*$", s)
    return False

def extract(code, lang):
    lines = code.split("\n")
    out = []
    i = 0
    while i < len(lines):
        line = lines[i]
        if lang == "python":
            if indent(line) == 0 and re.match(r"(async\s+)?def\s+\w", line.strip()):
                depth = 0; k = i
                while k < len(lines):
                    depth += lines[k].count("(") - lines[k].count(")")
                    if depth <= 0 and lines[k].rstrip().endswith(":"): break
                    k += 1
                    if k - i > 8: break
                sig_end = k
                j = sig_end + 1; body = lines[i:sig_end + 1]
                while j < len(lines):
                    if lines[j].strip() == "": body.append(lines[j]); j += 1; continue
                    if indent(lines[j]) > 0: body.append(lines[j]); j += 1
                    else: break
                out.append((i + 1, j, "\n".join(body))); i = j; continue
        else:
            if indent(line) == 0 and sig(line, lang):
                depth = 0; started = False; j = i; body = []
                while j < len(lines):
                    body.append(lines[j])
                    for ch in lines[j]:
                        if ch == "{": depth += 1; started = True
                        elif ch == "}": depth -= 1
                    if started and depth <= 0: break
                    j += 1
                    if j - i > 60: break
                if started and depth <= 0:
                    out.append((i + 1, j + 1, "\n".join(body))); i = j + 1; continue
        i += 1
    return out

def ok(text):
    if not (MIN_CHARS <= len(text) <= MAX_CHARS): return False
    ls = text.split("\n")
    if len(ls) < 4 or len(ls) > MAX_LINES: return False
    if any(len(l) > MAX_LINE_LEN for l in ls): return False
    return True

def excluded(path):
    return any(e in path for e in EXCLUDE)

snippets = []
for lang, repos in REPOS.items():
    ext = EXT[lang]
    got = 0
    for repo in repos:
        try:
            branch = api(f"https://api.github.com/repos/{repo}")["default_branch"]
            tree = api(f"https://api.github.com/repos/{repo}/git/trees/{branch}?recursive=1")["tree"]
        except Exception as e:
            print(f"  ! {repo}: {e}", file=sys.stderr); continue
        paths = [t["path"] for t in tree if t["type"] == "blob"
                 and t["path"].endswith(ext) and not excluded(t["path"])]
        random.shuffle(paths)
        for path in paths[:FILES_PER_REPO]:
            if got >= PER_LANG_CAP: break
            try: src = raw(repo, branch, path)
            except Exception: continue
            for start, end, block in extract(src, lang):
                clean = normalize(strip_comments(block, lang))
                if ok(clean):
                    snippets.append({
                        "language": lang, "repo": repo, "path": path,
                        "url": f"https://github.com/{repo}/blob/{branch}/{path}#L{start}-L{end}",
                        "content": clean,
                    })
                    got += 1
                    if got >= PER_LANG_CAP: break
        print(f"{lang}: {got} snippets", file=sys.stderr)

random.shuffle(snippets)
print(json.dumps(snippets, indent=1))
print(f"TOTAL {len(snippets)} snippets", file=sys.stderr)
