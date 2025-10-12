# Documentation Cleanup and Creation Tasks

## ✅ Completed
- [x] Archive development/design docs to docs/archive/ (38 files archived)
- [x] Create archive README
- [x] Create Housing System Guide (docs/user-guide/housing.md) - VERIFIED & FIXED ✓
- [x] Create Economy & Trading Guide (docs/user-guide/economy.md) - VERIFIED & FIXED ✓
- [x] Move daemon docs to administration directory
- [x] Review and organize all remaining docs (30 active docs)
- [x] **Verify accuracy of all user guides against code** ✨ NEW ✓
  * economy.md - Fixed currency system documentation (commit 0f0bb86)
  * housing.md - Fixed to match one-time purchase implementation (commit 8e25f81)
  * games.md - Verified accurate (no changes needed)
  * commands.md - Verified accurate (no changes needed)
  * connecting.md - Verified accurate
  * message-topics.md - Verified accurate
  * troubleshooting.md - Verified accurate

## 📝 Remaining Tasks

### User Guides (Priority 1)
- [ ] Task 1: Create Companion Guide (docs/user-guide/companions.md) **[TinyMUSH ONLY]**
  * Taming, bonding, training
  * Companion commands (FEED, PET, STAY, COME, MOUNT)
  * Care and lifecycle
  * **Note**: Only needed if/when TinyMUSH features are enabled

- [ ] Task 2: Create Quest Guide (docs/user-guide/quests.md) **[TinyMUSH ONLY]**
  * Finding and accepting quests
  * Quest types and objectives
  * Rewards and progression
  * Quest commands
  * **Note**: Only needed if/when TinyMUSH features are enabled

- [x] Task 3: Update Commands Reference (docs/user-guide/commands.md) ✓
  * Commands.md is comprehensive and accurate
  * Covers BBS commands, games, moderation, admin
  * Verified against code implementation
  * **No TinyMUSH-specific commands to add at this time**

### Admin Guides (Priority 2)
- [ ] Task 4: Create World Building Guide (docs/administration/world-building.md) **[TinyMUSH ONLY]**
  * Room creation and editing
  * Object creation and cloning
  * Area management
  * Builder commands
  * **Note**: Only needed if/when TinyMUSH features are enabled

- [ ] Task 5: Create Backup & Recovery Guide (docs/administration/backup-recovery.md)
  * Manual backup commands (BACKUP command exists)
  * Automated backup configuration
  * Restoration procedures
  * Verification and management
  * **Priority**: Document existing backup system

- [ ] Task 6: Update Admin Commands Reference (docs/administration/commands.md)
  * Comprehensive admin command list
  * Permission levels
  * Examples and use cases
  * **Note**: May consolidate with user-guide/commands.md

### Documentation Structure (Priority 3)
- [ ] Task 7: Update docs/index.md
  * Reflect new guide structure
  * Add links to housing, economy, companions, quests
  * Update navigation

- [x] Task 8: Clean up redundant docs ✅
  * Reviewed all 30 remaining files
  * Moved daemon docs to proper location
  * Kept all relevant QA and development docs
  * No broken links found

## Progress Summary

### Documentation Verification (Session Oct 12, 2025)
**Goal**: Ensure all user documentation matches actual code implementation

**Results**:
- 2 files fixed (economy.md, housing.md)
- 5 files verified accurate (games.md, commands.md, connecting.md, message-topics.md, troubleshooting.md)
- 2 files skipped (TRIGGER_ENGINE_GUIDE.md - previously verified, TUTORIAL_WALKTHROUGH.md - procedural)
- 53 lines of unimplemented features removed from housing.md
- All core user documentation is now **production-ready** ✓

**Commits**:
- 0f0bb86: Fix economy.md currency system documentation
- 8e25f81: Fix housing.md to match actual implementation

## Notes
- **TinyMUSH features** (companions, quests, crafting, world-building) are optional
  * These guides should be created only if/when TinyMUSH is enabled for production
  * Current BBS functionality is fully documented and accurate
- Focus on practical examples in all guides
- Cross-reference related guides
- All guides should be user-facing (not development docs)
- **Core BBS documentation is complete and accurate**


