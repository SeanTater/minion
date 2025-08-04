# Developer Onboarding Documentation

Welcome to the Minion ARPG development team! This directory contains focused onboarding materials to help new developers understand the system architecture and development workflows.

## Quick Start

1. **Read the project overview** in the main CLAUDE.md file
2. **Follow setup instructions** in DEVELOPMENT_WORKFLOWS.md
3. **Understand core systems** using the guides below
4. **Reference troubleshooting** when you encounter issues

## Documentation Structure

### 🏗️ [ARCHITECTURE_DECISIONS.md](ARCHITECTURE_DECISIONS.md)
**Key technical decisions and their rationale**

Essential reading for understanding why the codebase works the way it does. Covers:
- Movement system architecture (KinematicCharacterController vs physics)
- Terrain generation choices (noise-functions, heightmaps)
- Physics integration strategy (Rapier)
- LOD system design
- Configuration and error handling approaches

### 🌍 [TERRAIN_SYSTEM_GUIDE.md](TERRAIN_SYSTEM_GUIDE.md)
**Complete guide to the terrain generation and rendering pipeline**

Everything you need to know about terrain:
- Using the mapgen CLI tool
- Understanding terrain algorithms and parameters
- Spawn zone intelligence system
- Terrain following for characters
- Performance targets and optimization
- Integration with physics and rendering

### 🏃 [MOVEMENT_TROUBLESHOOTING.md](MOVEMENT_TROUBLESHOOTING.md)
**Debugging character movement issues**

Step-by-step troubleshooting for movement problems:
- Common movement issues and solutions
- Debug logging and tools
- Systematic debugging protocol
- Fallback strategies when things go wrong
- Configuration reference

### ⚙️ [DEVELOPMENT_WORKFLOWS.md](DEVELOPMENT_WORKFLOWS.md)
**Day-to-day development practices and workflows**

Practical guidance for productive development:
- Build and run commands
- Map generation workflows
- Debugging strategies
- Testing approaches
- Asset management with Git LFS
- Code style and patterns
- Performance considerations

## Learning Path for New Developers

### Week 1: Core Understanding
1. **Setup environment** - Follow DEVELOPMENT_WORKFLOWS.md setup section
2. **Read architecture decisions** - Understand key technical choices
3. **Generate your first map** - Follow terrain system guide
4. **Run the game** - Build and test basic functionality

### Week 2: System Deep Dive
1. **Study terrain system** - Generate different map types, understand parameters
2. **Learn movement system** - Read troubleshooting guide, test character movement
3. **Practice debugging** - Use debug tools, enable logging, test edge cases
4. **Make small changes** - Tweak parameters, fix small bugs

### Week 3: Active Development
1. **Pick a feature** - Choose something that interests you
2. **Follow workflows** - Use established patterns and practices
3. **Test thoroughly** - Use unit tests, integration tests, manual testing
4. **Get feedback** - Share work, iterate based on review

## Key Concepts to Understand

### System Integration
- How terrain generation connects to physics and rendering
- How movement system integrates with terrain following
- How LOD system affects performance and visual quality

### Development Philosophy
- **Minimalism**: Prefer editing existing code over creating new files
- **Performance First**: Consider performance implications of decisions
- **Maintainability**: Write code that's easy to understand and modify

### Common Patterns
- Resource pools with phantom types for type safety
- Configuration with runtime validation and graceful degradation
- Error handling with custom types and helpful messages
- Testing with unit tests for logic, integration tests for workflows

## When You Need Help

### Troubleshooting Movement Issues
→ Follow the systematic protocol in MOVEMENT_TROUBLESHOOTING.md

### Understanding a Technical Decision
→ Check ARCHITECTURE_DECISIONS.md for rationale and trade-offs

### Learning a New Workflow
→ Reference the step-by-step guides in DEVELOPMENT_WORKFLOWS.md

### Terrain Generation Questions
→ Use TERRAIN_SYSTEM_GUIDE.md for comprehensive coverage

### Still Stuck?
- Check the main CLAUDE.md for project overview
- Review recent commits for examples of good changes
- Ask questions with specific error messages and debug output

## Contributing to Documentation

These documents should stay focused on onboarding needs. When updating:

1. **Keep it tutorial-oriented** - Focus on what developers need to do
2. **Remove obsolete information** - Don't keep historical development notes
3. **Maintain clear structure** - Use consistent formatting and organization
4. **Test your documentation** - Ensure instructions actually work

The goal is to help new team members become productive quickly while understanding the architectural decisions that shape the codebase.

Welcome to the team! 🎮
