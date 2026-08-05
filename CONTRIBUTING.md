# Contributing

## Before contributing

Read:

- `ARCHITECTURE.md`;
- `SECURITY.md`;
- `THREAT_MODEL.md`;
- `TESTING.md`.

## Setup

```bash
npm install
anchor build
anchor test
```

## Expectations

Contributions must:

- preserve deterministic behaviour;
- include tests;
- describe security impact;
- avoid secrets;
- avoid generated dependencies;
- update documentation;
- use clear commit messages.

## Commit style

Recommended:

```text
feat: ...
fix: ...
security: ...
test: ...
docs: ...
chore: ...
```

## Pull request checklist

- [ ] Purpose explained
- [ ] On-chain behaviour described
- [ ] Security impact described
- [ ] Account-layout impact described
- [ ] Tests added
- [ ] Local tests pass
- [ ] Documentation updated
- [ ] No secrets or generated files
- [ ] Migration plan included when needed

## Security contributions

Do not open a public pull request containing an unpatched exploit against a live deployment.
