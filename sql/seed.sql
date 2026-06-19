-- Ship model seed data (from config/ships/*.toml)
INSERT INTO ship_models (id, size, role, price, base_shield, base_armor, base_energy, base_speed, base_agility, shields_count, armors_count, capacitors_count, motors_count, computers_count) VALUES
('550e8400-e29b-41d4-a716-446655440001', 'Frigate',     'Engineer',  40000, 250.0, 150.0, 200.0, 70.0,  0.3,  2, 1, 1, 0, 1),
('550e8400-e29b-41d4-a716-446655440002', 'Frigate',     'LongRange', 45000, 150.0, 100.0, 250.0, 85.0,  0.4,  1, 1, 2, 0, 1),
('550e8400-e29b-41d4-a716-446655440003', 'Frigate',     'Guard',     50000, 200.0, 300.0, 150.0, 60.0,  0.2,  1, 2, 1, 0, 1),
('550e8400-e29b-41d4-a716-446655440004', 'Fighter',    'Tackler',   35000, 120.0, 100.0, 180.0, 110.0, 0.7,  1, 1, 1, 1, 0),
('550e8400-e29b-41d4-a716-446655440005', 'Fighter',    'Gunship',   38000, 160.0, 140.0, 160.0, 95.0,  0.5,  1, 1, 1, 0, 1),
('550e8400-e29b-41d4-a716-446655440006', 'Fighter',    'Command',   42000, 180.0, 120.0, 220.0, 90.0,  0.5,  1, 1, 1, 0, 1),
('550e8400-e29b-41d4-a716-446655440007', 'Interceptor','CoverOps',  32000, 80.0,  60.0,  200.0, 130.0, 0.9,  1, 1, 1, 1, 0),
('550e8400-e29b-41d4-a716-446655440008', 'Interceptor','Recon',     30000, 90.0,  70.0,  220.0, 120.0, 0.8,  1, 0, 1, 0, 2),
('550e8400-e29b-41d4-a716-446655440009', 'Interceptor','Ecm',       34000, 100.0, 50.0,  250.0, 125.0, 0.85, 1, 0, 2, 0, 1);
