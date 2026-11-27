# Pinned Upstream & Local Engine Lineage (`vendor/`)

Upstream source repositories are pinned by exact Git commit SHA in `VENDOR_MANIFEST.json` (`A24`: large third-party source trees are never committed directly into this repo's Git object store).

To materialize all pinned upstream repositories locally into `vendor/<owner>__<repo>`:
```powershell
$m = Get-Content -LiteralPath 'vendor\VENDOR_MANIFEST.json' -Raw | ConvertFrom-Json
foreach ($u in $m.upstream_github) {
  $dir = 'vendor\' + ($u.repo -replace '/', '__')
  if (-not (Test-Path -LiteralPath $dir)) {
    git clone --depth 1 $u.url $dir
  }
}
```

Companion bare-metal A-Life engines authored by Alexander Stephen Donahue (`autophage`, `cellular-engine-omega`, `cellulogonic-mecha`, `noetic-organism-omega`, `planaria`, `alife-singularity-omega-package`) are optionally linked locally via their flat C11 ABIs (`autophage.dll`, `omega_cell.dll`, `cellulogonic.dll`, etc.).
