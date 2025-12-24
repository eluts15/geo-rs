### Debug

The simulation stuff needs elevated access..
```
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER='sudo -E' cargo test
```

Output:
```
test gpio_input::tests::test_heading_wraparound ... FAILED
test gpio_input::tests::test_toggle_switch_positions ... FAILED
test gpio_input::tests::test_multiple_adjustments ... FAILED

failures:

---- gpio_input::tests::test_heading_wraparound stdout ----
Error: UnknownModel

---- gpio_input::tests::test_toggle_switch_positions stdout ----
Error: UnknownModel

---- gpio_input::tests::test_multiple_adjustments stdout ----
Error: UnknownModel


failures:
    gpio_input::tests::test_heading_wraparound
    gpio_input::tests::test_multiple_adjustments
    gpio_input::tests::test_toggle_switch_positions

test result: FAILED. 8 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```


