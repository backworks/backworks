# Refactoring Plan for Backworks

## Completed Tasks
- Removed mock.rs module
- Changed default execution mode to `proxy`
- Updated EndpointConfig to remove mock fields
- Deprecated MockConfig and MockResponse structs
- Added documentation for the mock-free architecture

## Remaining Tasks

### Main Issues:

1. **Fix engine.rs**:
   - Import ExecutionMode from config
   - Remove mock and mock_responses fields from EndpointConfig creation
   - Check for ai field usage and replace with plugins

2. **Fix database.rs**:
   - Update DatabaseConfig struct field usage
   - Replace old field names (database_type → db_type)
   - Fix property access for database connections

3. **Fix capture.rs**:
   - Update CaptureConfig field usage
   - Fix Option<bool> vs bool type mismatches
   - Remove unused variables in tests

4. **Fix dashboard.rs**:
   - Update DashboardConfig field access

5. **Fix runtime.rs**:
   - Fix assertion in tests to use .is_ok() properly

### General Tasks:
- Run tests after each major fix
- Add proper field prefixes (_config) for unused variables
- Update examples to use proxy mode instead of mock
- Update documentation to reflect new architecture

### Testing Approach:
1. Fix compilation errors one module at a time
2. Run cargo check frequently
3. Run individual tests for each module after fixing
4. Run the full test suite after all fixes

## Timeline
1. Fix critical compilation errors (engine.rs, config.rs references)
2. Fix module-specific errors (database.rs, capture.rs)
3. Fix test cases
4. Update examples and documentation
5. Final validation and testing
