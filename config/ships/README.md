# Ship Model Configs

Per-model ship configuration files (TOML) defining base stats and passive module slot counts.

Each file defines one ship model with:
- Base stats (shield, armor, energy, speed, agility)
- Passive module slot counts per type (shields_count, armors_count, capacitors_count, motors_count, computers_count)
- Role assignment

Loaded at server startup to seed the `ship_models` table when a database URL is provided.
