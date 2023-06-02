mod infra;

// Your tests go here!
success_tests! {
    {
        name: make_vec_succ,
        file: "make_vec.snek",
        input: "5",
        expected: "[0, 0, 0, 0, 0]",
    },
    {
        name: vec_succ,
        file: "vec.snek",
        expected: "[0, 1, 2, 3]",
    },
    {
        name: vec_get_succ,
        file: "vec_get.snek",
        input: "3",
        expected: "3",
    },
    {
        name: linked_list_manipulations,
        file: "linked_list_manipulations.snek",
        expected: "1\n2\n3\n4\n5\n5\n4\n3\n2\n1\nnil"
    },
    {
        name: make_vecs_succ_5,
        file: "make_vecs.snek",
        input: "5",
        expected: "[]\n[1]\n[2, 2]\n[3, 3, 3]\n[4, 4, 4, 4]\n[5, 5, 5, 5, 5]",
    },
    {
        name: make_vecs_5_succ_0,
        file: "make_vecs.snek",
        input: "0",
        heap_size: 5,
        expected: "[]",
    },
    {
        name: make_vecs_5_succ_1,
        file: "make_vecs.snek",
        input: "1",
        heap_size: 5,
        expected: "[]\n[1]",
    },
    {
        name: make_vecs_5_succ_2,
        file: "make_vecs.snek",
        input: "2",
        heap_size: 5,
        expected: "[]\n[1]\n[2, 2]",
    },
    {
        name: make_vecs_5_succ_3,
        file: "make_vecs.snek",
        input: "3",
        heap_size: 5,
        expected: "[]\n[1]\n[2, 2]\n[3, 3, 3]",
    },
    {
        name: make_vecs_20_succ_5,
        file: "make_vecs.snek",
        input: "5",
        heap_size: 20,
        expected: "[]\n[1]\n[2, 2]\n[3, 3, 3]\n[4, 4, 4, 4]\n[5, 5, 5, 5, 5]",
    },
    {
        name: make_linked_lists_5_60_succ,
        file: "make_linked_lists.snek",
        input: "5",
        heap_size: 60,
        expected: "nil
[0, nil]
[0, [1, nil]]
[0, [1, [2, nil]]]
[0, [1, [2, [3, nil]]]]
[0, [1, [2, [3, [4, nil]]]]]"
    },
    {
        name: make_linked_lists_5_20_succ,
        file: "make_linked_lists.snek",
        input: "5",
        heap_size: 20,
        expected: "nil
[0, nil]
[0, [1, nil]]
[0, [1, [2, nil]]]
[0, [1, [2, [3, nil]]]]
[0, [1, [2, [3, [4, nil]]]]]"
    },
    {
        name: alternating_gc_5_60_succ,
        file: "alternating_gc.snek",
        input: "5",
        heap_size: 60,
        expected: "nil
[0, nil]
[0, [1, nil]]
[0, [1, [2, nil]]]
[0, [1, [2, [3, nil]]]]
[0, [1, [2, [3, [4, nil]]]]]"
    },
    {
        name: alternating_gc_5_20_succ,
        file: "alternating_gc.snek",
        input: "5",
        heap_size: 20,
        expected: "nil
[0, nil]
[0, [1, nil]]
[0, [1, [2, nil]]]
[0, [1, [2, [3, nil]]]]
[0, [1, [2, [3, [4, nil]]]]]"
    },
    {
        name: cleanup_nested_succ,
        file: "cleanup_nested.snek",
        input: "1000",
        heap_size: 4008,
        expected: "1000",
    },
    {
        name: multiple_cycles_succ,
        file: "multiple_cycles.snek",
        input: "1",
        heap_size: 7,
        expected: "[10, 9, 11, [...], [...]]
[9, 8, 10, [...], [...]]
[8, 7, 9, [...], [...]]
[7, 6, 8, [...], [...]]
[6, 5, 7, [...], [...]]
[5, 4, 6, [...], [...]]
[4, 3, 5, [...], [...]]
[3, 2, 4, [...], [...]]
[2, 1, 3, [...], [...]]
[1, 0, 2, [...], [...]]
nil"
    },
    {
        name: multiple_stack_references_succ,
        file: "multiple_stack_references.snek",
        input: "false",
        heap_size: 8,
        expected: "[5, nil]
[5, nil]
[6, nil]
[6, nil]
[4, nil]
[4, nil]
[5, nil]
[5, nil]
[3, nil]
[3, nil]
[4, nil]
[4, nil]
[2, nil]
[2, nil]
[3, nil]
[3, nil]
[1, nil]
[1, nil]
[2, nil]
[2, nil]
nil"
    },
}

runtime_error_tests! {
    {
        name: make_vec_oom,
        file: "make_vec.snek",
        input: "5",
        heap_size: 5,
        expected: "out of memory",
    },
    {
        name: vec_get_oob,
        file: "vec_get.snek",
        input: "5",
        expected: "",
    },
    {
        name: make_vecs_5_oom_4,
        file: "make_vecs.snek",
        input: "4",
        heap_size: 5,
        expected: "out of memory",
    },
    {
        name: make_vecs_5_oom_5,
        file: "make_vecs.snek",
        input: "5",
        heap_size: 5,
        expected: "out of memory",
    },
    {
        name: make_linked_lists_5_19_oom,
        file: "make_linked_lists.snek",
        input: "5",
        heap_size: 19,
        expected: "out of memory",
    },
    {
        name: alternating_gc_5_19_oom,
        file: "alternating_gc.snek",
        input: "5",
        heap_size: 19,
        expected: "out of memory",
    },
    {
        name: cleanup_nested_oom,
        file: "cleanup_nested.snek",
        input: "1000",
        heap_size: 4007,
        expected: "out of memory",
    },
    {
        name: multiple_cycles_oom,
        file: "multiple_cycles.snek",
        input: "1",
        heap_size: 6,
        expected: "out of memory",
    },
    {
        name: multiple_stack_references_oom,
        file: "multiple_stack_references.snek",
        input: "false",
        heap_size: 7,
        expected: "out of memory",
    },
}

static_error_tests! {}
