# Changelog

All notable changes to Commandy will be documented in this file.

## [0.1.0] - 2024-07-24

### 🚀 Major Features Added

#### Caching System
- **Cache Criteria**: Only cache suggestions after 5+ uses with >70% success rate
- **Success Tracking**: Track `use_count`, `success_count`, and `success_rate` for each suggestion
- **Cache Analytics**: View cache statistics with `commandy config`
- **Time-based Expiry**: Cache entries expire after 7 days to stay relevant
- **Weighted Ranking**: Suggestions ranked by `(success_rate * 0.6 + confidence * 0.4)`

#### Executable Validation
- **Real-time Validation**: Use `which` command to validate executables exist in PATH
- **System Scanning**: Auto-detect available tools from `/usr/local/bin`, `/usr/bin`, `/bin`
- **Pseudo-command Rejection**: Reject API-style and pseudo-commands (e.g., `memgraph query`)
- **Learning**: Learn valid executables and update COMMANDY.md
- **Adaptation**: Adapt to each user's specific tool set

#### Enhanced User Interface
- **New Escape Key Behavior**: 
  - Single Escape → Follow-up/modify command
  - Double Escape → Exit to static view
- **Improved Prompts**: Clear instructions show "Esc=follow-up, Esc Esc=exit"
- **Spinner Indicators**: Visual feedback during AI processing, initialization, diagnostics
- **F Key Alternative**: Keep 'F' as alternative follow-up key for compatibility

#### Shell History Integration
- **History Parsing**: Parse `~/.bash_history` and `~/.zsh_history` for context
- **Relevance Filtering**: Only include contextually relevant commands
- **Smart Deduplication**: Merge shell history with commandy history intelligently
- **Keyword Matching**: Match command keywords with prompt keywords for better context

#### Learning System
- **Success Pattern Learning**: Record successful command patterns in COMMANDY.md
- **Executable Discovery**: Learn and validate new executables over time  
- **Context Categorization**: Organize learning by categories (Git, Docker, etc.)
- **Feedback Loop**: Continuous improvement based on execution success/failure

### 🔧 Technical Improvements

#### Enhanced AI Prompting
- **Stricter Validation Rules**: Enforce real executable requirements in system prompt
- **COMMANDY.md Integration**: Include learned patterns in AI context
- **Dynamic Tool Lists**: Show actually available tools instead of hardcoded lists
- **Better Error Handling**: Improved error messages and validation feedback

#### Database Schema Updates
- **New Columns**: Added `success_count` and `success_rate` to suggestions table
- **Automatic Migration**: Seamless database migration for existing users
- **Unique Constraints**: Prevent duplicate suggestion entries
- **Performance Indexes**: Optimized queries for cache retrieval

#### Code Quality
- **Clippy Compliance**: Fixed all clippy warnings for better code quality
- **Error Handling**: Improved error handling throughout the codebase
- **Logging**: Enhanced debug logging for troubleshooting
- **Documentation**: Comprehensive inline documentation

### 🐛 Bug Fixes
- **Command Execution Tracking**: Fixed command success/failure tracking
- **Cache Invalidation**: Proper cache invalidation for failed commands
- **Terminal Mode**: Fixed terminal mode restoration after interactive selection
- **Memory Usage**: Optimized memory usage for large shell histories

### 📚 Documentation Updates
- **Updated README**: Comprehensive feature documentation with examples
- **API Documentation**: Enhanced code documentation and examples
- **Architecture Guide**: Updated system architecture documentation
- **Context Management**: Detailed smart caching and learning documentation

### ⚡ Performance Improvements
- **Faster Cache Lookups**: Optimized database queries with proper indexing
- **Reduced Memory Usage**: Limit shell history parsing to relevant commands
- **Concurrent Operations**: Parallel processing for multiple operations
- **Efficient Validation**: Cache executable validation results

## [0.0.1] - 2024-07-23

### Initial Release
- Basic natural language to command translation
- Local AI model integration via Ollama
- Simple caching mechanism
- Interactive command selection
- Basic COMMANDY.md learning system
- Cross-platform support (macOS, Linux, Windows)