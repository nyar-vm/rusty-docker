# Rusty Docker - Oak Dockerfile Deep Integration Verification Checklist

- [ ] Checkpoint 1: Dockerfile AST parsing works correctly for valid Dockerfiles
- [ ] Checkpoint 2: Dockerfile parsing provides clear errors for invalid Dockerfiles
- [ ] Checkpoint 3: AST walker processes instructions in correct order
- [ ] Checkpoint 4: All standard Dockerfile instructions are supported
- [ ] Checkpoint 5: Instructions are executed according to Dockerfile semantics
- [ ] Checkpoint 6: Files are correctly copied from build context
- [ ] Checkpoint 7: .dockerignore patterns are respected
- [ ] Checkpoint 8: Existing docker build command works with new implementation
- [ ] Checkpoint 9: CLI integration is seamless
- [ ] Checkpoint 10: Errors are caught and reported correctly
- [ ] Checkpoint 11: Error messages are clear and helpful
- [ ] Checkpoint 12: All tests pass
- [ ] Checkpoint 13: Implementation works as expected with real-world Dockerfiles