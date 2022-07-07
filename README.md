# ![Butter](butter_text_only.svg)

> **⚠ Note:** This project is currently paused.

---

[Documents](./doc/README.md) | [Contributing Guidelines](./CONTRIBUTING.md)

A tasty language for building efficient software.

**Note:** Still work in progress and experimental. [Contributions] are welcome.

[contributions]: ./CONTRIBUTING.md

## A small taste

```butter
-- reverses an array in place
reverse(mut arr) => {
    len = arr^.len;
    for i in [0.< len // 2] {
        mut elem = &arr^[i];
        mut opposite = &arr^[len - i - 1];
        elem^, opposite^ <- >opposite^, >elem^;
    }
}
```

## Goals

Butter is a personal and experimental language that seeks balance for these aspects:

**Note:** Being an experimental language, these are all subject to change

- Concise: The language constructs should be simple and have a feel of scripting language.
- Explicit: There should be little-to-no vagueness syntax-wise nor semantic-wise.
- High-level: Low-level concepts that are hard to understand should be abstracted.
- Efficient: The added runtime code for compiled programs should be minimal both in size and runtime impact.
- Correct: Detectable errors should be caught on compile-time.

I also to want to experiment with novel features deemed necessary for these goals such refinement types.

Being my personal project, designs and features are ultimately up for my decision and taste. Some features can help with some aspect while also hurt other, this is where I weigh in the pros and cons. Of course, this doesn't mean I won't listen to suggestions, I can be naive on these decisions, I'll be happy to hear your thoughts about Butter's design by opening an issue.

## Road map

![A road map for the Butter programming language, depicted in the form style a retro video game map. A single road snakes from right-to-left, with multiple stops, ending with a causeway leading to a castle in the middle of a lake. A flag is placed on the first stop. The project mascot (a cute yellow bear) is at the second stop and exclaims "Let's Go!". The stops on the road are: parser, Hindley-Milner with row polymorphism, IR, lifetime analysis, refinement types, and finally LLVM. An enormous dragon looms behind the castle. The whole picture have a dark overlay with text "game paused" in the center](./roadmap.webp)

[More details](https://github.com/neverRare/butter/projects/1)

## Planned features

**Note:** These are subject to change

Features to be implemented

- Hindley-Milner type inference and checking
- Structural typing with row polymorphism
- Mix of ownership systems and automatic reference counting &mdash; data that are immutable and never moved are shareable, otherwise they are owned
- Reference types with "no shared mutable" rule
- Mutability/Shareability polymorphism of references
- Lifetime inference and analysis
- Refinement types

Features to be implemented later on

- Type annotation and type aliases
- Traits or type classes
- Module and visibility system
- `newtype` for nominally typed record types
- Shareable and interiorly mutable containers &mdash; this is an escape hatch for "no shared mutable" rule of reference types
- Low-level representation heuristics &mdash; as an example, the compiler will try to infer if such array can be just a stack array or if it needs to be allocated on heap. Refinement type is used to check if such array exceeds certain capacity
