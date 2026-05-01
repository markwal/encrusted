# npm audit notes

## Current findings

`npm audit` reports 3 moderate vulnerabilities. They all come from the same dev-only dependency path:

```text
webpack-dev-server -> sockjs -> uuid@8.3.2
```

The advisory is for `uuid <14.0.0`: missing buffer bounds checks in the v3, v5, and v6 UUID functions when a caller passes a `buf` argument.

## Why we are deferring a code change

The vulnerable dependency is only installed through `webpack-dev-server`, which is used by the local development server. It is not part of the production web bundle or Electron app runtime.

The installed `sockjs@0.3.24` code calls `uuid.v4()` without passing a buffer:

```js
uuidv4 = require('uuid').v4;
this.id = uuidv4();
```

That means the known vulnerable buffer path is not used by this project through the current dependency chain.

`sockjs` does not currently have a newer release than `0.3.24`, and `webpack-dev-server@5.2.3` still depends on `sockjs`. npm's suggested audit fix is to downgrade `webpack-dev-server` to `1.16.5`, which would be a major regression and is not an appropriate remediation.

## Plan

Keep the current dependency set for now and accept the dev-only audit warning.

Revisit this when one of these becomes available:

- `webpack-dev-server` releases a version that removes `sockjs`.
- `sockjs` releases a version that depends on `uuid >=14.0.0`.
- A maintained webpack dev server replacement becomes worth adopting for this repo.

Avoid replacing `webpack-dev-server` with a custom `webpack-dev-middleware` server unless the audit finding becomes more urgent or the project needs custom dev-server behavior anyway.
