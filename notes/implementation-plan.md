# Implementation Plan: Procedural Terrain Generation

## Overview
Detailed implementation plan for Stage 6 procedural terrain generation, broken down into actionable tickets that can be assigned to developers.

## Prerequisites
- Stage 5 (Terrain mesh generation) is complete ✓
- Existing TerrainData structure supports height grids ✓
- MapGen binary has working CLI infrastructure ✓

## Implementation Phases

### Phase 1: Core Noise Integration

#### Ticket 1.1: Add Noise Library Dependency
**Estimated Time:** 2 hours
**Priority:** High

**Tasks:**
1. Add `noise-functions = "0.4"` to Cargo.toml
2. Update Cargo.lock and verify compilation
3. Create basic integration test to ensure library works
4. Document dependency choice in notes/

**Acceptance Criteria:**
- Library compiles without errors
- Basic noise function call works
- No breaking changes to existing functionality

**Files to Modify:**
- `/home/sean-gallagher/sandbox/minion/Cargo.toml`

#### Ticket 1.2: Create Terrain Generator Module
**Estimated Time:** 6 hours
**Priority:** High

**Tasks:**
1. Create `src/terrain/generator.rs`
2. Implement `TerrainGenerator` struct and `TerrainAlgorithm` enum
3. Implement basic Perlin noise generation
4. Add unit tests for noise generation consistency
5. Integrate with existing `TerrainData::new()` workflow

**Acceptance Criteria:**
- `TerrainGenerator::generate()` produces valid TerrainData
- Generated heights are deterministic for same seed
- Height values are reasonable (no NaN/infinite)
- Unit tests pass

**Files to Create:**
- `/home/sean-gallagher/sandbox/minion/src/terrain/generator.rs`

**Files to Modify:**
- `/home/sean-gallagher/sandbox/minion/src/terrain/mod.rs`

#### Ticket 1.3: Extend MapGen CLI Basic Parameters
**Estimated Time:** 4 hours
**Priority:** High

**Tasks:**
1. Add seed, amplitude, frequency, octaves parameters to Args struct
2. Implement parameter validation
3. Update main() to use TerrainGenerator instead of create_flat()
4. Add helpful error messages for invalid parameters
5. Update CLI help text

**Acceptance Criteria:**
- CLI accepts new parameters without breaking existing functionality
- Parameter validation catches invalid inputs
- Generated terrain reflects parameter changes
- Help text is clear and informative

**Files to Modify:**
- `/home/sean-gallagher/sandbox/minion/src/bin/mapgen.rs`

#### Ticket 1.4: Integration Testing & Validation
**Estimated Time:** 3 hours
**Priority:** Medium

**Tasks:**
1. Test full pipeline: CLI → TerrainGenerator → TerrainData → Game loading
2. Verify mesh generation works with procedural terrain
3. Test with different parameter combinations
4. Validate terrain heights are game-appropriate

**Acceptance Criteria:**
- Generated maps load correctly in game
- Terrain mesh renders properly
- Physics collider works with procedural terrain
- No crashes or visual artifacts

### Phase 2: Terrain Presets & Advanced Algorithms

#### Ticket 2.1: Implement Terrain Presets
**Estimated Time:** 4 hours
**Priority:** Medium

**Tasks:**
1. Create preset system with flat, rolling, hills, mountains, archipelago
2. Add --preset parameter to CLI
3. Implement preset → TerrainGenerator conversion
4. Add preset documentation and examples

**Acceptance Criteria:**
- All presets generate visually distinct terrain
- Presets override individual parameters appropriately
- CLI examples work as documented
- Preset terrain is suitable for gameplay

**Files to Modify:**
- `/home/sean-gallagher/sandbox/minion/src/terrain/generator.rs`
- `/home/sean-gallagher/sandbox/minion/src/bin/mapgen.rs`

#### Ticket 2.2: Add Ridged and Layered Noise
**Estimated Time:** 5 hours
**Priority:** Medium

**Tasks:**
1. Implement Ridged noise algorithm in TerrainAlgorithm enum
2. Implement Layered algorithm for combining noise types
3. Add domain warping support if available in noise-functions
4. Update presets to use new algorithms where appropriate

**Acceptance Criteria:**
- Ridged noise produces mountain-like ridges
- Layered noise combines base + detail effectively
- New algorithms integrate seamlessly with existing system
- Visual results match expected terrain types

**Files to Modify:**
- `/home/sean-gallagher/sandbox/minion/src/terrain/generator.rs`

#### Ticket 2.3: Performance Optimization
**Estimated Time:** 4 hours
**Priority:** Low

**Tasks:**
1. Profile terrain generation performance
2. Optimize hot paths in noise generation
3. Add progress feedback for large terrain generation
4. Implement generation time estimates

**Acceptance Criteria:**
- 64x64 terrain generates in < 500ms
- 256x256 terrain generates in < 5s
- Progress feedback shows for long operations
- No memory leaks or excessive allocations

**Files to Modify:**
- `/home/sean-gallagher/sandbox/minion/src/terrain/generator.rs`
- `/home/sean-gallagher/sandbox/minion/src/bin/mapgen.rs`

### Phase 3: Biome System & Spawn Intelligence

#### Ticket 3.1: Implement Biome Classification
**Estimated Time:** 6 hours
**Priority:** Medium

**Tasks:**
1. Create `src/terrain/biomes.rs` module
2. Implement BiomeMap generation based on height and moisture
3. Add BiomeType enum (Plains, Hills, Mountains, Water, Swamp)
4. Integrate biome generation with terrain generation

**Acceptance Criteria:**
- Biomes are classified logically based on terrain features
- Biome boundaries are smooth and natural-looking
- Water biomes correctly identify low-elevation areas
- System integrates without breaking existing functionality

**Files to Create:**
- `/home/sean-gallagher/sandbox/minion/src/terrain/biomes.rs`

**Files to Modify:**
- `/home/sean-gallagher/sandbox/minion/src/terrain/mod.rs`

#### Ticket 3.2: Implement Intelligent Spawn Placement
**Estimated Time:** 8 hours
**Priority:** High

**Tasks:**
1. Create `src/terrain/spawn_placement.rs` module
2. Implement slope calculation and terrain analysis
3. Add spawn suitability scoring system
4. Replace ring-based spawn placement with terrain-aware algorithm
5. Add spawn placement validation and fallback strategies

**Acceptance Criteria:**
- Spawn zones avoid steep slopes and water
- Spawn zones maintain minimum distance requirements
- Algorithm handles cases where ideal placement is impossible
- Generated spawn zones are accessible from player spawn

**Files to Create:**
- `/home/sean-gallagher/sandbox/minion/src/terrain/spawn_placement.rs`

**Files to Modify:**
- `/home/sean-gallagher/sandbox/minion/src/bin/mapgen.rs`
- `/home/sean-gallagher/sandbox/minion/src/terrain/mod.rs`

#### Ticket 3.3: Spawn Placement CLI Integration
**Estimated Time:** 3 hours
**Priority:** Medium

**Tasks:**
1. Add spawn placement parameters to CLI (max_spawn_slope, min_spawn_distance, force_spawns)
2. Integrate spawn placement with mapgen workflow
3. Add validation for spawn placement parameters
4. Update help text and examples

**Acceptance Criteria:**
- CLI parameters control spawn placement behavior
- Parameter validation prevents invalid configurations
- Spawn placement integrates smoothly with terrain generation
- Error messages guide users when spawn placement fails

**Files to Modify:**
- `/home/sean-gallagher/sandbox/minion/src/bin/mapgen.rs`

### Phase 4: Debug Tools & Polish

#### Ticket 4.1: Debug Visualization Tools
**Estimated Time:** 5 hours
**Priority:** Low

**Tasks:**
1. Add --debug-heightmap flag to generate PNG heightmap
2. Add --debug-biomes flag to generate biome visualization
3. Implement image generation using simple image library
4. Add debug output for spawn placement decisions

**Acceptance Criteria:**
- Debug images accurately represent generated terrain
- Images are saved to maps/ directory with appropriate names
- Debug output helps users understand generation decisions
- Debug features don't affect normal operation

**Files to Modify:**
- `/home/sean-gallagher/sandbox/minion/src/bin/mapgen.rs`
- `/home/sean-gallagher/sandbox/minion/Cargo.toml` (add image dependency)

#### Ticket 4.2: Error Handling & Validation
**Estimated Time:** 4 hours
**Priority:** Medium

**Tasks:**
1. Implement comprehensive terrain validation
2. Add graceful fallback for failed noise generation
3. Improve error messages throughout the pipeline
4. Add recovery strategies for spawn placement failures

**Acceptance Criteria:**
- System handles edge cases gracefully
- Error messages are helpful and actionable
- No panics or crashes from invalid inputs
- Fallback strategies maintain functionality

**Files to Modify:**
- `/home/sean-gallagher/sandbox/minion/src/terrain/generator.rs`
- `/home/sean-gallagher/sandbox/minion/src/terrain/spawn_placement.rs`
- `/home/sean-gallagher/sandbox/minion/src/bin/mapgen.rs`

#### Ticket 4.3: Documentation & Examples
**Estimated Time:** 3 hours
**Priority:** Low

**Tasks:**
1. Create comprehensive examples in CLI help
2. Add inline documentation to all public APIs
3. Create example command workflows
4. Update project documentation with new capabilities

**Acceptance Criteria:**
- All public APIs have clear documentation
- Examples cover common use cases
- Documentation is accurate and up-to-date
- Users can follow examples successfully

**Files to Modify:**
- Various source files (inline documentation)
- `/home/sean-gallagher/sandbox/minion/src/bin/mapgen.rs` (help text)

## Implementation Strategy

### 1. Development Order
**Sequential Dependencies:**
- Phase 1 must complete before Phase 2
- Ticket 3.1 must complete before 3.2
- All core functionality before debug tools

**Parallel Development:**
- Tickets within same phase can be developed concurrently
- Debug tools (Phase 4) can start after Phase 2

### 2. Testing Strategy
**Per-Ticket Testing:**
- Unit tests for each new module
- Integration tests for CLI changes
- Visual validation for terrain generation

**End-to-End Testing:**
- Full mapgen → game loading pipeline
- Performance benchmarking
- Error condition handling

### 3. Risk Mitigation
**Technical Risks:**
- noise-functions performance issues → fallback to noise-rs (documented in research)
- Complex terrain breaking physics → validation and limits
- Spawn placement edge cases → fallback strategies

**Timeline Risks:**
- Phase 1 (core functionality) is highest priority
- Debug tools can be deferred if needed
- Biome system is nice-to-have, not critical

## Total Estimated Timeline
- **Phase 1:** 15 hours (3-4 days for one developer)
- **Phase 2:** 13 hours (3 days)
- **Phase 3:** 17 hours (4-5 days)
- **Phase 4:** 12 hours (3 days)

**Total:** 57 hours (~14 days for one developer, or ~7 days for two developers working in parallel)

## Success Criteria
1. **Functional:** mapgen generates visually appealing, gameplay-suitable terrain
2. **Usable:** CLI is approachable for non-experts with good defaults
3. **Robust:** System handles edge cases and provides helpful error messages
4. **Performant:** Terrain generation completes in reasonable time
5. **Integrated:** Generated maps work seamlessly with existing game systems

This implementation plan provides clear, actionable tickets that can be assigned and tracked independently while building toward the complete procedural terrain generation system.