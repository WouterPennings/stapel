; X86_64 Assembly linux (Ubuntu 22.04)
; NASM version 2.15.05
global _start

section .bss
    ; Reserve space for the global variables
    argc: resq 1   ; 64-bit integer
    argv: resq 1   ; 64-bit pointer

section .data
    ori_stack_ptr: dq 0 ; Pointer to start of stack
    ret_stack: TIMES 1024 DQ 0; Stack for the return adresses
    ret_stack_cursor: DQ 0; Pointer to start of memory

    overflow_msg: db "Error: Shadow Stack Overflow!", 10
    overflow_len: equ $ - overflow_msg

    underflow_msg: db "Error: Shadow Stack Underflow!", 10
    underflow_len: equ $ - underflow_msg

section .text
print_i64:
    sub     rsp, 40
    mov     rax, rdi
    mov     r9, 0               ; Sign flag
    ; --- Handle Negative ---
    test    rax, rax
    jns     .L2
    neg     rax
    mov     r9, 1
.L2:
    mov     BYTE [rsp+31], 10   ; Newline at the end
    lea     rsi, [rsp+30]       ; Buffer pointer
    mov     rcx, 0              ; Digit counter
    mov     r8, 10              ; Divisor
.L3:
    xor     rdx, rdx            ; Clear RDX for division
    div     r8                  ; RAX / 10 -> RAX (quotient), RDX (remainder)
    add     dl, 48              ; Convert to ASCII
    mov     [rsi], dl           ; Store digit
    dec     rsi                 ; Move pointer left
    inc     rcx                 ; Count digit
    test    rax, rax            ; Check if we have more digits to process
    jnz     .L3                 ; If RAX != 0, loop again
    ; --- Add Negative Sign ---
    test    r9, r9
    jz      .L4
    mov     BYTE [rsi], '-'
    dec     rsi
    inc     rcx
.L4:
    mov     rax, 1              ; sys_write
    mov     rdi, 1              ; stdout
    inc     rsi                 ; Pointer to the first character
    mov     rdx, rcx            ; Number of digits
    inc     rdx                 ; +1 for the newline
    syscall

    add     rsp, 40
    ret

call_proxy:
    ;pop rax            ; Get return address from hardware stack
    ;pop rdi            ; Get target procedure address from hardware stack

    ; --- Safety Check ---
    cmp r13, 1023      ; Check if we are at the limit of ret_stack (1024 entries)
    jge stack_overflow ; If r13 >= 1023, trigger error

    ; Use R13 as the index. 
    ; We scale by 8 because these are 64-bit (8-byte) addresses.
    inc r13                             ; Move to next slot
    mov [ret_stack + r13 * 8], rax      ; Store return address

    jmp rdi            ; Jump to procedure

stack_overflow:
    mov rax, 1          ; sys_write
    mov rdi, 2          ; stderr
    mov rsi, overflow_msg
    mov rdx, overflow_len
    syscall

    mov rax, 60        ; sys_exit
    mov rdi, 1         ; error code 1
    syscall

stack_underflow:
    mov rax, 1          ; sys_write
    mov rdi, 2          ; stderr
    mov rsi, underflow_msg
    mov rdx, underflow_len
    syscall

    mov rax, 60        ; sys_exit
    mov rdi, 1         ; error code 1
    syscall

_start:
    ; DEFAULT INSTRUCTIONS
    mov [ori_stack_ptr], rsp
    xor r13, r13

    ; --- CAPTURE ARGS ---
    mov rax, [rsp]      ; The top of the stack holds 'argc'
    mov [argc], rax     ; Save it to our global variable

    lea rax, [rsp + 8]  ; The next item is the pointer to argv[0]
    mov [argv], rax     ; Save the *address* of the argv array

    jmp proc_main
    
    ; ADDED COMPILED INSTRUCTIONS


proc_main:
	; --- Custom(malloc_heap_init) ---
	mov rdi, proc_malloc_heap_init
	mov rax, .addr_1
	jmp call_proxy
.addr_1:
	; --- PushStr("hello") ---
	push 5
	push str_0
	; --- Custom(input) ---
	push input
	; --- Custom(memcpy) ---
	mov rdi, proc_memcpy
	mov rax, .addr_2
	jmp call_proxy
.addr_2:
	; --- Custom(lexer) ---
	mov rdi, proc_lexer
	mov rax, .addr_3
	jmp call_proxy
.addr_3:
	; --- Custom(tokens) ---
	push tokens
	; --- Load(8) ---
	pop rax
	xor rbx, rbx
	mov rbx, [rax]
	push rbx
	; --- Put ---
	pop rdi
	call print_i64
	; --- PushStr("TokenCount: ") ---
	push 12
	push str_1
	; --- Custom(print) ---
	mov rdi, proc_print
	mov rax, .addr_4
	jmp call_proxy
.addr_4:
	; --- Custom(token_count) ---
	push token_count
	; --- Load(8) ---
	pop rax
	xor rbx, rbx
	mov rbx, [rax]
	push rbx
	; --- PushInt(24) ---
	mov rax, 24
	push rax
	; --- InfixOperator(Divide) ---
	pop rbx
	pop rax
	xor rdx, rdx
	idiv rbx
	push rax
	; --- Put ---
	pop rdi
	call print_i64
	; === GLOBAL EXIT ===
	mov rax, 60
	mov rdi, 0
	syscall


proc_exit:
	; --- Custom(sys_exit_nr) ---
	; --- PushInt(60) ---
	mov rax, 60
	push rax
	; --- Syscall(1) ---
	pop rax
	syscall
	push rax
	; --- Return ---
	test r13, r13
	jz stack_underflow
	mov rdx, [ret_stack + r13 * 8]
	dec r13
	jmp rdx

proc_memcpy:
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- While ---
.addr_5:
	; --- PushInt(3) ---
	mov rax, 3
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- PushInt(1) ---
	mov rax, 1
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- InfixOperator(GreaterThan) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	setg cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_6
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	; --- Store(1) ---
	pop rbx
	pop rax
	mov [rax], bl
	; --- PushInt(1) ---
	mov rax, 1
	push rax
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	jmp .addr_5
.addr_6:
	; --- Pop ---
	pop rax
	; --- Pop ---
	pop rax
	; --- Pop ---
	pop rax
	; --- Pop ---
	pop rax
	; --- Return ---
	test r13, r13
	jz stack_underflow
	mov rdx, [ret_stack + r13 * 8]
	dec r13
	jmp rdx

proc_itoa:
	; --- If ---
	; --- Dup ---
	pop rax
	push rax
	push rax
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- InfixOperator(Equals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	sete cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_8
	; --- PushInt(1) ---
	mov rax, 1
	push rax
	; --- InfixOperator(Minus) ---
	pop rbx
	pop rax
	sub rax, rbx
	push rax
	; --- Dup ---
	pop rax
	push rax
	push rax
	; --- PushInt(48) ---
	mov rax, 48
	push rax
	; --- Store(1) ---
	pop rbx
	pop rax
	mov [rax], bl
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	; --- Pop ---
	pop rax
	; --- PushInt(1) ---
	mov rax, 1
	push rax
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	; --- Return ---
	test r13, r13
	jz stack_underflow
	mov rdx, [ret_stack + r13 * 8]
	dec r13
	jmp rdx
	jmp .addr_7
.addr_8:
.addr_7:
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	; --- PushInt(32) ---
	mov rax, 32
	push rax
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- Store(1) ---
	pop rbx
	pop rax
	mov [rax], bl
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- While ---
.addr_9:
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- InfixOperator(GreaterThan) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	setg cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_10
	; --- PushInt(1) ---
	mov rax, 1
	push rax
	; --- InfixOperator(Minus) ---
	pop rbx
	pop rax
	sub rax, rbx
	push rax
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- PushInt(10) ---
	mov rax, 10
	push rax
	; --- InfixOperator(Modulo) ---
	pop rbx
	pop rax
	xor rdx, rdx
	cqo
	idiv rbx
	push rdx
	; --- PushInt(48) ---
	mov rax, 48
	push rax
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	; --- Store(1) ---
	pop rbx
	pop rax
	mov [rax], bl
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	; --- PushInt(10) ---
	mov rax, 10
	push rax
	; --- InfixOperator(Divide) ---
	pop rbx
	pop rax
	xor rdx, rdx
	idiv rbx
	push rax
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	jmp .addr_9
.addr_10:
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	; --- Pop ---
	pop rax
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- InfixOperator(Minus) ---
	pop rbx
	pop rax
	sub rax, rbx
	push rax
	; --- Rot ---
	pop rcx
	pop rbx
	pop rax
	push rbx
	push rcx
	push rax
	; --- Pop ---
	pop rax
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	; --- Return ---
	test r13, r13
	jz stack_underflow
	mov rdx, [ret_stack + r13 * 8]
	dec r13
	jmp rdx

proc_parse_word:
	; --- Custom(current_span) ---
	mov rdi, proc_current_span
	mov rax, .addr_11
	jmp call_proxy
.addr_11:
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- While ---
.addr_12:
	; --- Custom(current_char) ---
	; --- Custom(input) ---
	push input
	; --- Custom(cursor) ---
	push cursor
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(32) ---
	mov rax, 32
	push rax
	; --- InfixOperator(NotEquals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	setne cl
	push rcx
	; --- Custom(current_char) ---
	; --- Custom(input) ---
	push input
	; --- Custom(cursor) ---
	push cursor
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(10) ---
	mov rax, 10
	push rax
	; --- InfixOperator(NotEquals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	setne cl
	push rcx
	; --- InfixOperator(And) ---
	pop rbx
	pop rax
	cmp rax, 0
	setne al
	cmp rbx, 0
	setne bl
	and al, bl
	movzx rax, al
	push rax
	; --- Custom(current_char) ---
	; --- Custom(input) ---
	push input
	; --- Custom(cursor) ---
	push cursor
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- InfixOperator(NotEquals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	setne cl
	push rcx
	; --- InfixOperator(And) ---
	pop rbx
	pop rax
	cmp rax, 0
	setne al
	cmp rbx, 0
	setne bl
	and al, bl
	movzx rax, al
	push rax
	pop rax
	cmp rax, 0
	je .addr_13
	; --- Custom(current_char) ---
	; --- Custom(input) ---
	push input
	; --- Custom(cursor) ---
	push cursor
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- Custom(word_buffer) ---
	push word_buffer
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	; --- Store(1) ---
	pop rbx
	pop rax
	mov [rax], bl
	; --- Custom(next_character) ---
	mov rdi, proc_next_character
	mov rax, .addr_14
	jmp call_proxy
.addr_14:
	; --- Pop ---
	pop rax
	; --- PushInt(1) ---
	mov rax, 1
	push rax
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	jmp .addr_12
.addr_13:
	; --- Custom(word_buffer) ---
	push word_buffer
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- Store(1) ---
	pop rbx
	pop rax
	mov [rax], bl
	; --- Custom(word_buffer) ---
	push word_buffer
	; --- Custom(to_str) ---
	; --- Dup ---
	pop rax
	push rax
	push rax
	; --- Custom(strlen) ---
	mov rdi, proc_strlen
	mov rax, .addr_15
	jmp call_proxy
.addr_15:
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	; --- If ---
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- PushStr("and") ---
	push 3
	push str_2
	; --- Custom(streq) ---
	mov rdi, proc_streq
	mov rax, .addr_18
	jmp call_proxy
.addr_18:
	pop rax
	cmp rax, 0
	je .addr_17
	; --- Custom(tokentype_infix) ---
	; --- PushInt(3) ---
	mov rax, 3
	push rax
	; --- Custom(infix_and) ---
	; --- PushInt(12) ---
	mov rax, 12
	push rax
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_19
	jmp call_proxy
.addr_19:
	jmp .addr_16
.addr_17:
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- PushStr("or") ---
	push 2
	push str_3
	; --- Custom(streq) ---
	mov rdi, proc_streq
	mov rax, .addr_21
	jmp call_proxy
.addr_21:
	pop rax
	cmp rax, 0
	je .addr_20
	; --- Custom(tokentype_infix) ---
	; --- PushInt(3) ---
	mov rax, 3
	push rax
	; --- Custom(infix_or) ---
	; --- PushInt(13) ---
	mov rax, 13
	push rax
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_22
	jmp call_proxy
.addr_22:
	jmp .addr_16
.addr_20:
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- PushStr("pop") ---
	push 3
	push str_4
	; --- Custom(streq) ---
	mov rdi, proc_streq
	mov rax, .addr_24
	jmp call_proxy
.addr_24:
	pop rax
	cmp rax, 0
	je .addr_23
	; --- Custom(tokentype_pop) ---
	; --- PushInt(4) ---
	mov rax, 4
	push rax
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_25
	jmp call_proxy
.addr_25:
	jmp .addr_16
.addr_23:
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- PushStr("swap") ---
	push 4
	push str_5
	; --- Custom(streq) ---
	mov rdi, proc_streq
	mov rax, .addr_27
	jmp call_proxy
.addr_27:
	pop rax
	cmp rax, 0
	je .addr_26
	; --- Custom(tokentype_swap) ---
	; --- PushInt(5) ---
	mov rax, 5
	push rax
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_28
	jmp call_proxy
.addr_28:
	jmp .addr_16
.addr_26:
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- PushStr("rot") ---
	push 3
	push str_6
	; --- Custom(streq) ---
	mov rdi, proc_streq
	mov rax, .addr_30
	jmp call_proxy
.addr_30:
	pop rax
	cmp rax, 0
	je .addr_29
	; --- Custom(tokentype_rot) ---
	; --- PushInt(6) ---
	mov rax, 6
	push rax
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_31
	jmp call_proxy
.addr_31:
	jmp .addr_16
.addr_29:
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- PushStr("over") ---
	push 4
	push str_7
	; --- Custom(streq) ---
	mov rdi, proc_streq
	mov rax, .addr_33
	jmp call_proxy
.addr_33:
	pop rax
	cmp rax, 0
	je .addr_32
	; --- Custom(tokentype_over) ---
	; --- PushInt(7) ---
	mov rax, 7
	push rax
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_34
	jmp call_proxy
.addr_34:
	jmp .addr_16
.addr_32:
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- PushStr("pick") ---
	push 4
	push str_8
	; --- Custom(streq) ---
	mov rdi, proc_streq
	mov rax, .addr_36
	jmp call_proxy
.addr_36:
	pop rax
	cmp rax, 0
	je .addr_35
	; --- Custom(tokentype_pick) ---
	; --- PushInt(8) ---
	mov rax, 8
	push rax
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_37
	jmp call_proxy
.addr_37:
	jmp .addr_16
.addr_35:
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- PushStr("put") ---
	push 3
	push str_9
	; --- Custom(streq) ---
	mov rdi, proc_streq
	mov rax, .addr_39
	jmp call_proxy
.addr_39:
	pop rax
	cmp rax, 0
	je .addr_38
	; --- Custom(tokentype_put) ---
	; --- PushInt(9) ---
	mov rax, 9
	push rax
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_40
	jmp call_proxy
.addr_40:
	jmp .addr_16
.addr_38:
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- PushStr("if") ---
	push 2
	push str_10
	; --- Custom(streq) ---
	mov rdi, proc_streq
	mov rax, .addr_42
	jmp call_proxy
.addr_42:
	pop rax
	cmp rax, 0
	je .addr_41
	; --- Custom(tokentype_if) ---
	; --- PushInt(11) ---
	mov rax, 11
	push rax
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_43
	jmp call_proxy
.addr_43:
	jmp .addr_16
.addr_41:
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- PushStr("elif") ---
	push 4
	push str_11
	; --- Custom(streq) ---
	mov rdi, proc_streq
	mov rax, .addr_45
	jmp call_proxy
.addr_45:
	pop rax
	cmp rax, 0
	je .addr_44
	; --- Custom(tokentype_elif) ---
	; --- PushInt(12) ---
	mov rax, 12
	push rax
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_46
	jmp call_proxy
.addr_46:
	jmp .addr_16
.addr_44:
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- PushStr("else") ---
	push 4
	push str_12
	; --- Custom(streq) ---
	mov rdi, proc_streq
	mov rax, .addr_48
	jmp call_proxy
.addr_48:
	pop rax
	cmp rax, 0
	je .addr_47
	; --- Custom(tokentype_else) ---
	; --- PushInt(13) ---
	mov rax, 13
	push rax
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_49
	jmp call_proxy
.addr_49:
	jmp .addr_16
.addr_47:
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- PushStr("end") ---
	push 3
	push str_13
	; --- Custom(streq) ---
	mov rdi, proc_streq
	mov rax, .addr_51
	jmp call_proxy
.addr_51:
	pop rax
	cmp rax, 0
	je .addr_50
	; --- Custom(tokentype_end) ---
	; --- PushInt(15) ---
	mov rax, 15
	push rax
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_52
	jmp call_proxy
.addr_52:
	jmp .addr_16
.addr_50:
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- PushStr("do") ---
	push 2
	push str_14
	; --- Custom(streq) ---
	mov rdi, proc_streq
	mov rax, .addr_54
	jmp call_proxy
.addr_54:
	pop rax
	cmp rax, 0
	je .addr_53
	; --- Custom(tokentype_do) ---
	; --- PushInt(14) ---
	mov rax, 14
	push rax
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_55
	jmp call_proxy
.addr_55:
	jmp .addr_16
.addr_53:
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- PushStr("while") ---
	push 5
	push str_15
	; --- Custom(streq) ---
	mov rdi, proc_streq
	mov rax, .addr_57
	jmp call_proxy
.addr_57:
	pop rax
	cmp rax, 0
	je .addr_56
	; --- Custom(tokentype_while) ---
	; --- PushInt(10) ---
	mov rax, 10
	push rax
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_58
	jmp call_proxy
.addr_58:
	jmp .addr_16
.addr_56:
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- PushStr("dup") ---
	push 3
	push str_16
	; --- Custom(streq) ---
	mov rdi, proc_streq
	mov rax, .addr_60
	jmp call_proxy
.addr_60:
	pop rax
	cmp rax, 0
	je .addr_59
	; --- Custom(tokentype_dup) ---
	; --- PushInt(16) ---
	mov rax, 16
	push rax
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_61
	jmp call_proxy
.addr_61:
	jmp .addr_16
.addr_59:
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- PushStr("size") ---
	push 4
	push str_17
	; --- Custom(streq) ---
	mov rdi, proc_streq
	mov rax, .addr_63
	jmp call_proxy
.addr_63:
	pop rax
	cmp rax, 0
	je .addr_62
	; --- Custom(tokentype_size) ---
	; --- PushInt(17) ---
	mov rax, 17
	push rax
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_64
	jmp call_proxy
.addr_64:
	jmp .addr_16
.addr_62:
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- PushStr("memory") ---
	push 6
	push str_18
	; --- Custom(streq) ---
	mov rdi, proc_streq
	mov rax, .addr_66
	jmp call_proxy
.addr_66:
	pop rax
	cmp rax, 0
	je .addr_65
	; --- Custom(tokentype_memory) ---
	; --- PushInt(18) ---
	mov rax, 18
	push rax
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_67
	jmp call_proxy
.addr_67:
	jmp .addr_16
.addr_65:
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- PushStr("return") ---
	push 6
	push str_19
	; --- Custom(streq) ---
	mov rdi, proc_streq
	mov rax, .addr_69
	jmp call_proxy
.addr_69:
	pop rax
	cmp rax, 0
	je .addr_68
	; --- Custom(tokentype_return) ---
	; --- PushInt(19) ---
	mov rax, 19
	push rax
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_70
	jmp call_proxy
.addr_70:
	jmp .addr_16
.addr_68:
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- PushStr("proc") ---
	push 4
	push str_20
	; --- Custom(streq) ---
	mov rdi, proc_streq
	mov rax, .addr_72
	jmp call_proxy
.addr_72:
	pop rax
	cmp rax, 0
	je .addr_71
	; --- Custom(tokentype_proc) ---
	; --- PushInt(20) ---
	mov rax, 20
	push rax
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_73
	jmp call_proxy
.addr_73:
	jmp .addr_16
.addr_71:
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- PushStr("inline") ---
	push 6
	push str_21
	; --- Custom(streq) ---
	mov rdi, proc_streq
	mov rax, .addr_75
	jmp call_proxy
.addr_75:
	pop rax
	cmp rax, 0
	je .addr_74
	; --- Custom(tokentype_inline) ---
	; --- PushInt(21) ---
	mov rax, 21
	push rax
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_76
	jmp call_proxy
.addr_76:
	jmp .addr_16
.addr_74:
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	; --- Pop ---
	pop rax
	; --- Custom(tokentype_ident) ---
	; --- PushInt(25) ---
	mov rax, 25
	push rax
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_77
	jmp call_proxy
.addr_77:
.addr_16:
	; --- Pop ---
	pop rax
	; --- Return ---
	test r13, r13
	jz stack_underflow
	mov rdx, [ret_stack + r13 * 8]
	dec r13
	jmp rdx

proc_malloc:
	; --- Custom(malloc_heap_ptr) ---
	push malloc_heap_ptr
	; --- Load(8) ---
	pop rax
	xor rbx, rbx
	mov rbx, [rax]
	push rbx
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Dup ---
	pop rax
	push rax
	push rax
	; --- Custom(sys_brk_nr) ---
	; --- PushInt(12) ---
	mov rax, 12
	push rax
	; --- Syscall(2) ---
	pop rax
	pop rdi
	syscall
	push rax
	; --- Custom(malloc_heap_ptr) ---
	push malloc_heap_ptr
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	; --- Store(8) ---
	pop rbx
	pop rax
	mov [rax], rbx
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	; --- Pop ---
	pop rax
	; --- Return ---
	test r13, r13
	jz stack_underflow
	mov rdx, [ret_stack + r13 * 8]
	dec r13
	jmp rdx

proc_sys_write:
	; --- Custom(sys_write_nr) ---
	; --- PushInt(1) ---
	mov rax, 1
	push rax
	; --- Syscall(4) ---
	pop rax
	pop rdi
	pop rsi
	pop rdx
	syscall
	push rax
	; --- Return ---
	test r13, r13
	jz stack_underflow
	mov rdx, [ret_stack + r13 * 8]
	dec r13
	jmp rdx

proc_println:
	; --- Custom(print) ---
	mov rdi, proc_print
	mov rax, .addr_78
	jmp call_proxy
.addr_78:
	; --- PushStr("\n") ---
	push 1
	push str_22
	; --- Custom(print) ---
	mov rdi, proc_print
	mov rax, .addr_79
	jmp call_proxy
.addr_79:
	; --- Return ---
	test r13, r13
	jz stack_underflow
	mov rdx, [ret_stack + r13 * 8]
	dec r13
	jmp rdx

proc_sys_open:
	; --- Custom(sys_open_nr) ---
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Syscall(3) ---
	pop rax
	pop rdi
	pop rsi
	syscall
	push rax
	; --- Return ---
	test r13, r13
	jz stack_underflow
	mov rdx, [ret_stack + r13 * 8]
	dec r13
	jmp rdx

proc_dump_stack:
	; --- PushStr("======================") ---
	push 22
	push str_23
	; --- Custom(println) ---
	mov rdi, proc_println
	mov rax, .addr_80
	jmp call_proxy
.addr_80:
	; --- PushStr("Stack size: ") ---
	push 12
	push str_24
	; --- Custom(print) ---
	mov rdi, proc_print
	mov rax, .addr_81
	jmp call_proxy
.addr_81:
	; --- Size ---
	mov rax, rsp
	sub rax, [ori_stack_ptr]
	neg rax
	shr rax, 3
	push rax
	; --- Put ---
	pop rdi
	call print_i64
	; --- While ---
.addr_82:
	; --- Size ---
	mov rax, rsp
	sub rax, [ori_stack_ptr]
	neg rax
	shr rax, 3
	push rax
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- InfixOperator(NotEquals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	setne cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_83
	; --- PushStr(" - ") ---
	push 3
	push str_25
	; --- Custom(print) ---
	mov rdi, proc_print
	mov rax, .addr_84
	jmp call_proxy
.addr_84:
	; --- Put ---
	pop rdi
	call print_i64
	jmp .addr_82
.addr_83:
	; --- PushStr("======================") ---
	push 22
	push str_26
	; --- Custom(println) ---
	mov rdi, proc_println
	mov rax, .addr_85
	jmp call_proxy
.addr_85:
	; --- Return ---
	test r13, r13
	jz stack_underflow
	mov rdx, [ret_stack + r13 * 8]
	dec r13
	jmp rdx

proc_streq:
	; --- If ---
	; --- PushInt(3) ---
	mov rax, 3
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- InfixOperator(NotEquals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	setne cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_87
	; --- Pop ---
	pop rax
	; --- Pop ---
	pop rax
	; --- Pop ---
	pop rax
	; --- Pop ---
	pop rax
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	jmp .addr_86
.addr_87:
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	; --- Pop ---
	pop rax
	; --- Rot ---
	pop rcx
	pop rbx
	pop rax
	push rbx
	push rcx
	push rax
	; --- While ---
.addr_88:
	; --- Dup ---
	pop rax
	push rax
	push rax
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- InfixOperator(GreaterThan) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	setg cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_89
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Pick ---
	pop rax
	shl rax, 3
	mov rbx, [rsp + rax]
	push rbx
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- If ---
	; --- InfixOperator(NotEquals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	setne cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_91
	; --- Pop ---
	pop rax
	; --- Pop ---
	pop rax
	; --- Pop ---
	pop rax
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- Return ---
	test r13, r13
	jz stack_underflow
	mov rdx, [ret_stack + r13 * 8]
	dec r13
	jmp rdx
	jmp .addr_90
.addr_91:
.addr_90:
	; --- PushInt(1) ---
	mov rax, 1
	push rax
	; --- InfixOperator(Minus) ---
	pop rbx
	pop rax
	sub rax, rbx
	push rax
	; --- Rot ---
	pop rcx
	pop rbx
	pop rax
	push rbx
	push rcx
	push rax
	; --- PushInt(1) ---
	mov rax, 1
	push rax
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Rot ---
	pop rcx
	pop rbx
	pop rax
	push rbx
	push rcx
	push rax
	; --- PushInt(1) ---
	mov rax, 1
	push rax
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Rot ---
	pop rcx
	pop rbx
	pop rax
	push rbx
	push rcx
	push rax
	jmp .addr_88
.addr_89:
	; --- Pop ---
	pop rax
	; --- Pop ---
	pop rax
	; --- Pop ---
	pop rax
	; --- PushInt(1) ---
	mov rax, 1
	push rax
.addr_86:
	; --- Return ---
	test r13, r13
	jz stack_underflow
	mov rdx, [ret_stack + r13 * 8]
	dec r13
	jmp rdx

proc_print:
	; --- Custom(stdout) ---
	; --- PushInt(1) ---
	mov rax, 1
	push rax
	; --- Custom(sys_write_nr) ---
	; --- PushInt(1) ---
	mov rax, 1
	push rax
	; --- Syscall(4) ---
	pop rax
	pop rdi
	pop rsi
	pop rdx
	syscall
	push rax
	; --- Pop ---
	pop rax
	; --- Return ---
	test r13, r13
	jz stack_underflow
	mov rdx, [ret_stack + r13 * 8]
	dec r13
	jmp rdx

proc_append_token:
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	; --- Rot ---
	pop rcx
	pop rbx
	pop rax
	push rbx
	push rcx
	push rax
	; --- Custom(tokens) ---
	push tokens
	; --- Custom(token_count) ---
	push token_count
	; --- Load(8) ---
	pop rax
	xor rbx, rbx
	mov rbx, [rax]
	push rbx
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	; --- Store(8) ---
	pop rbx
	pop rax
	mov [rax], rbx
	; --- Custom(tokens) ---
	push tokens
	; --- Custom(token_count) ---
	push token_count
	; --- Load(8) ---
	pop rax
	xor rbx, rbx
	mov rbx, [rax]
	push rbx
	; --- PushInt(8) ---
	mov rax, 8
	push rax
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	; --- Store(8) ---
	pop rbx
	pop rax
	mov [rax], rbx
	; --- Custom(tokens) ---
	push tokens
	; --- Custom(token_count) ---
	push token_count
	; --- Load(8) ---
	pop rax
	xor rbx, rbx
	mov rbx, [rax]
	push rbx
	; --- PushInt(16) ---
	mov rax, 16
	push rax
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	; --- Store(8) ---
	pop rbx
	pop rax
	mov [rax], rbx
	; --- Custom(token_count) ---
	push token_count
	; --- Dup ---
	pop rax
	push rax
	push rax
	; --- Load(8) ---
	pop rax
	xor rbx, rbx
	mov rbx, [rax]
	push rbx
	; --- PushInt(24) ---
	mov rax, 24
	push rax
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Store(8) ---
	pop rbx
	pop rax
	mov [rax], rbx
	; --- Return ---
	test r13, r13
	jz stack_underflow
	mov rdx, [ret_stack + r13 * 8]
	dec r13
	jmp rdx

proc_current_span:
	; --- PushInt(24) ---
	mov rax, 24
	push rax
	; --- Custom(malloc) ---
	mov rdi, proc_malloc
	mov rax, .addr_92
	jmp call_proxy
.addr_92:
	; --- Dup ---
	pop rax
	push rax
	push rax
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Custom(file_name) ---
	push file_name
	; --- Load(8) ---
	pop rax
	xor rbx, rbx
	mov rbx, [rax]
	push rbx
	; --- Store(8) ---
	pop rbx
	pop rax
	mov [rax], rbx
	; --- Dup ---
	pop rax
	push rax
	push rax
	; --- PushInt(8) ---
	mov rax, 8
	push rax
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Custom(line_count) ---
	push line_count
	; --- Load(8) ---
	pop rax
	xor rbx, rbx
	mov rbx, [rax]
	push rbx
	; --- Store(8) ---
	pop rbx
	pop rax
	mov [rax], rbx
	; --- Dup ---
	pop rax
	push rax
	push rax
	; --- PushInt(16) ---
	mov rax, 16
	push rax
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Custom(column_count) ---
	push column_count
	; --- Load(8) ---
	pop rax
	xor rbx, rbx
	mov rbx, [rax]
	push rbx
	; --- Store(8) ---
	pop rbx
	pop rax
	mov [rax], rbx
	; --- Return ---
	test r13, r13
	jz stack_underflow
	mov rdx, [ret_stack + r13 * 8]
	dec r13
	jmp rdx

proc_strlen:
	; --- Dup ---
	pop rax
	push rax
	push rax
	; --- While ---
.addr_93:
	; --- Dup ---
	pop rax
	push rax
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- InfixOperator(NotEquals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	setne cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_94
	; --- PushInt(1) ---
	mov rax, 1
	push rax
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	jmp .addr_93
.addr_94:
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- InfixOperator(Minus) ---
	pop rbx
	pop rax
	sub rax, rbx
	push rax
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	; --- Pop ---
	pop rax
	; --- Return ---
	test r13, r13
	jz stack_underflow
	mov rdx, [ret_stack + r13 * 8]
	dec r13
	jmp rdx

proc_next_character:
	; --- Custom(cursor) ---
	push cursor
	; --- Dup ---
	pop rax
	push rax
	push rax
	; --- Load(8) ---
	pop rax
	xor rbx, rbx
	mov rbx, [rax]
	push rbx
	; --- PushInt(1) ---
	mov rax, 1
	push rax
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Store(8) ---
	pop rbx
	pop rax
	mov [rax], rbx
	; --- If ---
	; --- Custom(current_char) ---
	; --- Custom(input) ---
	push input
	; --- Custom(cursor) ---
	push cursor
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(10) ---
	mov rax, 10
	push rax
	; --- InfixOperator(Equals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	sete cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_96
	; --- Custom(line_count) ---
	push line_count
	; --- Dup ---
	pop rax
	push rax
	push rax
	; --- Load(8) ---
	pop rax
	xor rbx, rbx
	mov rbx, [rax]
	push rbx
	; --- PushInt(1) ---
	mov rax, 1
	push rax
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Store(8) ---
	pop rbx
	pop rax
	mov [rax], rbx
	; --- Custom(column_count) ---
	push column_count
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- Store(8) ---
	pop rbx
	pop rax
	mov [rax], rbx
	jmp .addr_95
.addr_96:
	; --- Custom(column_count) ---
	push column_count
	; --- Dup ---
	pop rax
	push rax
	push rax
	; --- Load(8) ---
	pop rax
	xor rbx, rbx
	mov rbx, [rax]
	push rbx
	; --- PushInt(1) ---
	mov rax, 1
	push rax
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Store(8) ---
	pop rbx
	pop rax
	mov [rax], rbx
.addr_95:
	; --- Custom(current_char) ---
	; --- Custom(input) ---
	push input
	; --- Custom(cursor) ---
	push cursor
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- Return ---
	test r13, r13
	jz stack_underflow
	mov rdx, [ret_stack + r13 * 8]
	dec r13
	jmp rdx

proc_atoi:
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- While ---
.addr_97:
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- InfixOperator(GreaterThan) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	setg cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_98
	; --- Rot ---
	pop rcx
	pop rbx
	pop rax
	push rbx
	push rcx
	push rax
	; --- Dup ---
	pop rax
	push rax
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(48) ---
	mov rax, 48
	push rax
	; --- InfixOperator(Minus) ---
	pop rbx
	pop rax
	sub rax, rbx
	push rax
	; --- Rot ---
	pop rcx
	pop rbx
	pop rax
	push rbx
	push rcx
	push rax
	; --- PushInt(10) ---
	mov rax, 10
	push rax
	; --- InfixOperator(Multiply) ---
	pop rbx
	pop rax
	xor rdx, rdx
	imul rbx
	push rax
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	; --- PushInt(1) ---
	mov rax, 1
	push rax
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Rot ---
	pop rcx
	pop rbx
	pop rax
	push rbx
	push rcx
	push rax
	; --- PushInt(1) ---
	mov rax, 1
	push rax
	; --- InfixOperator(Minus) ---
	pop rbx
	pop rax
	sub rax, rbx
	push rax
	; --- Rot ---
	pop rcx
	pop rbx
	pop rax
	push rbx
	push rcx
	push rax
	jmp .addr_97
.addr_98:
	; --- Rot ---
	pop rcx
	pop rbx
	pop rax
	push rbx
	push rcx
	push rax
	; --- Rot ---
	pop rcx
	pop rbx
	pop rax
	push rbx
	push rcx
	push rax
	; --- Pop ---
	pop rax
	; --- Pop ---
	pop rax
	; --- Return ---
	test r13, r13
	jz stack_underflow
	mov rdx, [ret_stack + r13 * 8]
	dec r13
	jmp rdx

proc_lexer:
	; --- Custom(cursor) ---
	push cursor
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- Store(8) ---
	pop rbx
	pop rax
	mov [rax], rbx
	; --- While ---
.addr_99:
	; --- Custom(current_char) ---
	; --- Custom(input) ---
	push input
	; --- Custom(cursor) ---
	push cursor
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- InfixOperator(NotEquals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	setne cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_100
	; --- While ---
.addr_101:
	; --- Custom(current_char) ---
	; --- Custom(input) ---
	push input
	; --- Custom(cursor) ---
	push cursor
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(32) ---
	mov rax, 32
	push rax
	; --- InfixOperator(Equals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	sete cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_102
	; --- Custom(next_character) ---
	mov rdi, proc_next_character
	mov rax, .addr_103
	jmp call_proxy
.addr_103:
	; --- Pop ---
	pop rax
	jmp .addr_101
.addr_102:
	; --- If ---
	; --- Custom(current_char) ---
	; --- Custom(input) ---
	push input
	; --- Custom(cursor) ---
	push cursor
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(59) ---
	mov rax, 59
	push rax
	; --- InfixOperator(Equals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	sete cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_105
	; --- While ---
.addr_106:
	; --- Custom(current_char) ---
	; --- Custom(input) ---
	push input
	; --- Custom(cursor) ---
	push cursor
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(10) ---
	mov rax, 10
	push rax
	; --- InfixOperator(NotEquals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	setne cl
	push rcx
	; --- Custom(current_char) ---
	; --- Custom(input) ---
	push input
	; --- Custom(cursor) ---
	push cursor
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- Custom(EOF) ---
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- InfixOperator(NotEquals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	setne cl
	push rcx
	; --- InfixOperator(And) ---
	pop rbx
	pop rax
	cmp rax, 0
	setne al
	cmp rbx, 0
	setne bl
	and al, bl
	movzx rax, al
	push rax
	pop rax
	cmp rax, 0
	je .addr_107
	; --- Custom(next_character) ---
	mov rdi, proc_next_character
	mov rax, .addr_108
	jmp call_proxy
.addr_108:
	; --- Pop ---
	pop rax
	jmp .addr_106
.addr_107:
	jmp .addr_104
.addr_105:
	; --- Custom(current_char) ---
	; --- Custom(input) ---
	push input
	; --- Custom(cursor) ---
	push cursor
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(34) ---
	mov rax, 34
	push rax
	; --- InfixOperator(Equals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	sete cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_109
	; --- PushStr("TODO: tokenizing of string") ---
	push 26
	push str_27
	; --- Custom(println) ---
	mov rdi, proc_println
	mov rax, .addr_110
	jmp call_proxy
.addr_110:
	jmp .addr_104
.addr_109:
	; --- Custom(current_char) ---
	; --- Custom(input) ---
	push input
	; --- Custom(cursor) ---
	push cursor
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(39) ---
	mov rax, 39
	push rax
	; --- InfixOperator(Equals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	sete cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_111
	; --- PushStr("TODO: tokenizing of char") ---
	push 24
	push str_28
	; --- Custom(println) ---
	mov rdi, proc_println
	mov rax, .addr_112
	jmp call_proxy
.addr_112:
	jmp .addr_104
.addr_111:
	; --- Custom(current_char) ---
	; --- Custom(input) ---
	push input
	; --- Custom(cursor) ---
	push cursor
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(43) ---
	mov rax, 43
	push rax
	; --- InfixOperator(Equals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	sete cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_113
	; --- Custom(tokentype_infix) ---
	; --- PushInt(3) ---
	mov rax, 3
	push rax
	; --- Custom(infix_add) ---
	; --- PushInt(1) ---
	mov rax, 1
	push rax
	; --- Custom(current_span) ---
	mov rdi, proc_current_span
	mov rax, .addr_114
	jmp call_proxy
.addr_114:
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_115
	jmp call_proxy
.addr_115:
	jmp .addr_104
.addr_113:
	; --- Custom(current_char) ---
	; --- Custom(input) ---
	push input
	; --- Custom(cursor) ---
	push cursor
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(45) ---
	mov rax, 45
	push rax
	; --- InfixOperator(Equals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	sete cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_116
	; --- Custom(tokentype_infix) ---
	; --- PushInt(3) ---
	mov rax, 3
	push rax
	; --- Custom(infix_sub) ---
	; --- PushInt(2) ---
	mov rax, 2
	push rax
	; --- Custom(current_span) ---
	mov rdi, proc_current_span
	mov rax, .addr_117
	jmp call_proxy
.addr_117:
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_118
	jmp call_proxy
.addr_118:
	jmp .addr_104
.addr_116:
	; --- Custom(current_char) ---
	; --- Custom(input) ---
	push input
	; --- Custom(cursor) ---
	push cursor
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(42) ---
	mov rax, 42
	push rax
	; --- InfixOperator(Equals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	sete cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_119
	; --- Custom(tokentype_infix) ---
	; --- PushInt(3) ---
	mov rax, 3
	push rax
	; --- Custom(infix_mul) ---
	; --- PushInt(3) ---
	mov rax, 3
	push rax
	; --- Custom(current_span) ---
	mov rdi, proc_current_span
	mov rax, .addr_120
	jmp call_proxy
.addr_120:
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_121
	jmp call_proxy
.addr_121:
	jmp .addr_104
.addr_119:
	; --- Custom(current_char) ---
	; --- Custom(input) ---
	push input
	; --- Custom(cursor) ---
	push cursor
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(47) ---
	mov rax, 47
	push rax
	; --- InfixOperator(Equals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	sete cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_122
	; --- Custom(tokentype_infix) ---
	; --- PushInt(3) ---
	mov rax, 3
	push rax
	; --- Custom(infix_div) ---
	; --- PushInt(4) ---
	mov rax, 4
	push rax
	; --- Custom(current_span) ---
	mov rdi, proc_current_span
	mov rax, .addr_123
	jmp call_proxy
.addr_123:
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_124
	jmp call_proxy
.addr_124:
	jmp .addr_104
.addr_122:
	; --- Custom(current_char) ---
	; --- Custom(input) ---
	push input
	; --- Custom(cursor) ---
	push cursor
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(37) ---
	mov rax, 37
	push rax
	; --- InfixOperator(Equals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	sete cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_125
	; --- Custom(tokentype_infix) ---
	; --- PushInt(3) ---
	mov rax, 3
	push rax
	; --- Custom(infix_mod) ---
	; --- PushInt(5) ---
	mov rax, 5
	push rax
	; --- Custom(current_span) ---
	mov rdi, proc_current_span
	mov rax, .addr_126
	jmp call_proxy
.addr_126:
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_127
	jmp call_proxy
.addr_127:
	jmp .addr_104
.addr_125:
	; --- Custom(current_char) ---
	; --- Custom(input) ---
	push input
	; --- Custom(cursor) ---
	push cursor
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(61) ---
	mov rax, 61
	push rax
	; --- InfixOperator(Equals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	sete cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_128
	; --- Custom(tokentype_infix) ---
	; --- PushInt(3) ---
	mov rax, 3
	push rax
	; --- Custom(infix_eq) ---
	; --- PushInt(6) ---
	mov rax, 6
	push rax
	; --- Custom(current_span) ---
	mov rdi, proc_current_span
	mov rax, .addr_129
	jmp call_proxy
.addr_129:
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_130
	jmp call_proxy
.addr_130:
	jmp .addr_104
.addr_128:
	; --- Custom(current_char) ---
	; --- Custom(input) ---
	push input
	; --- Custom(cursor) ---
	push cursor
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(60) ---
	mov rax, 60
	push rax
	; --- InfixOperator(Equals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	sete cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_131
	; --- If ---
	; --- Custom(peek_char) ---
	; --- Custom(input) ---
	push input
	; --- Custom(cursor) ---
	push cursor
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- PushInt(8) ---
	mov rax, 8
	push rax
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(61) ---
	mov rax, 61
	push rax
	; --- InfixOperator(Equals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	sete cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_133
	; --- Custom(tokentype_infix) ---
	; --- PushInt(3) ---
	mov rax, 3
	push rax
	; --- Custom(infix_le) ---
	; --- PushInt(11) ---
	mov rax, 11
	push rax
	; --- Custom(current_span) ---
	mov rdi, proc_current_span
	mov rax, .addr_134
	jmp call_proxy
.addr_134:
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_135
	jmp call_proxy
.addr_135:
	; --- Custom(next_character) ---
	mov rdi, proc_next_character
	mov rax, .addr_136
	jmp call_proxy
.addr_136:
	jmp .addr_132
.addr_133:
	; --- Custom(tokentype_infix) ---
	; --- PushInt(3) ---
	mov rax, 3
	push rax
	; --- Custom(infix_lt) ---
	; --- PushInt(8) ---
	mov rax, 8
	push rax
	; --- Custom(current_span) ---
	mov rdi, proc_current_span
	mov rax, .addr_137
	jmp call_proxy
.addr_137:
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_138
	jmp call_proxy
.addr_138:
.addr_132:
	jmp .addr_104
.addr_131:
	; --- Custom(current_char) ---
	; --- Custom(input) ---
	push input
	; --- Custom(cursor) ---
	push cursor
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(62) ---
	mov rax, 62
	push rax
	; --- InfixOperator(Equals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	sete cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_139
	; --- If ---
	; --- Custom(peek_char) ---
	; --- Custom(input) ---
	push input
	; --- Custom(cursor) ---
	push cursor
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- PushInt(8) ---
	mov rax, 8
	push rax
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(61) ---
	mov rax, 61
	push rax
	; --- InfixOperator(Equals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	sete cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_141
	; --- Custom(tokentype_infix) ---
	; --- PushInt(3) ---
	mov rax, 3
	push rax
	; --- Custom(infix_ge) ---
	; --- PushInt(10) ---
	mov rax, 10
	push rax
	; --- Custom(current_span) ---
	mov rdi, proc_current_span
	mov rax, .addr_142
	jmp call_proxy
.addr_142:
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_143
	jmp call_proxy
.addr_143:
	; --- Custom(next_character) ---
	mov rdi, proc_next_character
	mov rax, .addr_144
	jmp call_proxy
.addr_144:
	jmp .addr_140
.addr_141:
	; --- Custom(tokentype_infix) ---
	; --- PushInt(3) ---
	mov rax, 3
	push rax
	; --- Custom(infix_gt) ---
	; --- PushInt(9) ---
	mov rax, 9
	push rax
	; --- Custom(current_span) ---
	mov rdi, proc_current_span
	mov rax, .addr_145
	jmp call_proxy
.addr_145:
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_146
	jmp call_proxy
.addr_146:
.addr_140:
	jmp .addr_104
.addr_139:
	; --- Custom(current_char) ---
	; --- Custom(input) ---
	push input
	; --- Custom(cursor) ---
	push cursor
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(33) ---
	mov rax, 33
	push rax
	; --- InfixOperator(Equals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	sete cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_147
	; --- Custom(next_character) ---
	mov rdi, proc_next_character
	mov rax, .addr_148
	jmp call_proxy
.addr_148:
	; --- If ---
	; --- Custom(current_char) ---
	; --- Custom(input) ---
	push input
	; --- Custom(cursor) ---
	push cursor
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(61) ---
	mov rax, 61
	push rax
	; --- InfixOperator(Equals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	sete cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_150
	; --- Custom(tokentype_infix) ---
	; --- PushInt(3) ---
	mov rax, 3
	push rax
	; --- Custom(infix_nq) ---
	; --- PushInt(7) ---
	mov rax, 7
	push rax
	; --- Custom(current_span) ---
	mov rdi, proc_current_span
	mov rax, .addr_151
	jmp call_proxy
.addr_151:
	; --- Custom(append_token) ---
	mov rdi, proc_append_token
	mov rax, .addr_152
	jmp call_proxy
.addr_152:
	jmp .addr_149
.addr_150:
	; --- PushStr("TODO: tokenizing of loading") ---
	push 27
	push str_29
	; --- Custom(println) ---
	mov rdi, proc_println
	mov rax, .addr_153
	jmp call_proxy
.addr_153:
.addr_149:
	jmp .addr_104
.addr_147:
	; --- Custom(current_char) ---
	; --- Custom(input) ---
	push input
	; --- Custom(cursor) ---
	push cursor
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- InfixOperator(Plus) ---
	pop rbx
	pop rax
	add rax, rbx
	push rax
	; --- Load(1) ---
	pop rax
	xor rbx, rbx
	mov bl, [rax]
	push rbx
	; --- PushInt(64) ---
	mov rax, 64
	push rax
	; --- InfixOperator(Equals) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	sete cl
	push rcx
	pop rax
	cmp rax, 0
	je .addr_154
	; --- PushStr("TODO: tokenizing of storing") ---
	push 27
	push str_30
	; --- Custom(println) ---
	mov rdi, proc_println
	mov rax, .addr_155
	jmp call_proxy
.addr_155:
	jmp .addr_104
.addr_154:
	; --- Custom(parse_word) ---
	mov rdi, proc_parse_word
	mov rax, .addr_156
	jmp call_proxy
.addr_156:
.addr_104:
	; --- Custom(next_character) ---
	mov rdi, proc_next_character
	mov rax, .addr_157
	jmp call_proxy
.addr_157:
	jmp .addr_99
.addr_100:
	; --- Return ---
	test r13, r13
	jz stack_underflow
	mov rdx, [ret_stack + r13 * 8]
	dec r13
	jmp rdx

proc_malloc_heap_init:
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- Custom(sys_brk_nr) ---
	; --- PushInt(12) ---
	mov rax, 12
	push rax
	; --- Syscall(2) ---
	pop rax
	pop rdi
	syscall
	push rax
	; --- Custom(malloc_heap_ptr) ---
	push malloc_heap_ptr
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	; --- Store(8) ---
	pop rbx
	pop rax
	mov [rax], rbx
	; --- Return ---
	test r13, r13
	jz stack_underflow
	mov rdx, [ret_stack + r13 * 8]
	dec r13
	jmp rdx

proc_put_char:
	; --- Custom(char_buffer) ---
	push char_buffer
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	; --- Store(1) ---
	pop rbx
	pop rax
	mov [rax], bl
	; --- PushInt(1) ---
	mov rax, 1
	push rax
	; --- Custom(char_buffer) ---
	push char_buffer
	; --- Custom(print) ---
	mov rdi, proc_print
	mov rax, .addr_158
	jmp call_proxy
.addr_158:
	; --- Return ---
	test r13, r13
	jz stack_underflow
	mov rdx, [ret_stack + r13 * 8]
	dec r13
	jmp rdx

proc_sys_read:
	; --- Custom(sys_read_nr) ---
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- Syscall(4) ---
	pop rax
	pop rdi
	pop rsi
	pop rdx
	syscall
	push rax
	; --- Return ---
	test r13, r13
	jz stack_underflow
	mov rdx, [ret_stack + r13 * 8]
	dec r13
	jmp rdx

proc_read_byte:
	; --- Custom(char_buffer) ---
	push char_buffer
	; --- PushInt(1) ---
	mov rax, 1
	push rax
	; --- Swap ---
	pop rax
	pop rbx
	push rax
	push rbx
	; --- Rot ---
	pop rcx
	pop rbx
	pop rax
	push rbx
	push rcx
	push rax
	; --- Custom(sys_read) ---
	mov rdi, proc_sys_read
	mov rax, .addr_159
	jmp call_proxy
.addr_159:
	; --- Return ---
	test r13, r13
	jz stack_underflow
	mov rdx, [ret_stack + r13 * 8]
	dec r13
	jmp rdx

section .bss
	token_count: resb 8
	line_count: resb 8
	word_buffer: resb 256
	char_buffer: resb 1
	cursor: resb 8
	file_name: resb 8
	column_count: resb 8
	malloc_heap_ptr: resb 8
	input: resb 1024
	tokens: resb 2401

section .data
; Strings with null terminators
	str_0: db 0x68,0x65,0x6c,0x6c,0x6f, 0 ; "hello"
	str_1: db 0x54,0x6f,0x6b,0x65,0x6e,0x43,0x6f,0x75,0x6e,0x74,0x3a,0x20, 0 ; "TokenCount: "
	str_2: db 0x61,0x6e,0x64, 0 ; "and"
	str_3: db 0x6f,0x72, 0 ; "or"
	str_4: db 0x70,0x6f,0x70, 0 ; "pop"
	str_5: db 0x73,0x77,0x61,0x70, 0 ; "swap"
	str_6: db 0x72,0x6f,0x74, 0 ; "rot"
	str_7: db 0x6f,0x76,0x65,0x72, 0 ; "over"
	str_8: db 0x70,0x69,0x63,0x6b, 0 ; "pick"
	str_9: db 0x70,0x75,0x74, 0 ; "put"
	str_10: db 0x69,0x66, 0 ; "if"
	str_11: db 0x65,0x6c,0x69,0x66, 0 ; "elif"
	str_12: db 0x65,0x6c,0x73,0x65, 0 ; "else"
	str_13: db 0x65,0x6e,0x64, 0 ; "end"
	str_14: db 0x64,0x6f, 0 ; "do"
	str_15: db 0x77,0x68,0x69,0x6c,0x65, 0 ; "while"
	str_16: db 0x64,0x75,0x70, 0 ; "dup"
	str_17: db 0x73,0x69,0x7a,0x65, 0 ; "size"
	str_18: db 0x6d,0x65,0x6d,0x6f,0x72,0x79, 0 ; "memory"
	str_19: db 0x72,0x65,0x74,0x75,0x72,0x6e, 0 ; "return"
	str_20: db 0x70,0x72,0x6f,0x63, 0 ; "proc"
	str_21: db 0x69,0x6e,0x6c,0x69,0x6e,0x65, 0 ; "inline"
	str_22: db 0xa, 0 ; "\n"
	str_23: db 0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d, 0 ; "======================"
	str_24: db 0x53,0x74,0x61,0x63,0x6b,0x20,0x73,0x69,0x7a,0x65,0x3a,0x20, 0 ; "Stack size: "
	str_25: db 0x20,0x2d,0x20, 0 ; " - "
	str_26: db 0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d,0x3d, 0 ; "======================"
	str_27: db 0x54,0x4f,0x44,0x4f,0x3a,0x20,0x74,0x6f,0x6b,0x65,0x6e,0x69,0x7a,0x69,0x6e,0x67,0x20,0x6f,0x66,0x20,0x73,0x74,0x72,0x69,0x6e,0x67, 0 ; "TODO: tokenizing of string"
	str_28: db 0x54,0x4f,0x44,0x4f,0x3a,0x20,0x74,0x6f,0x6b,0x65,0x6e,0x69,0x7a,0x69,0x6e,0x67,0x20,0x6f,0x66,0x20,0x63,0x68,0x61,0x72, 0 ; "TODO: tokenizing of char"
	str_29: db 0x54,0x4f,0x44,0x4f,0x3a,0x20,0x74,0x6f,0x6b,0x65,0x6e,0x69,0x7a,0x69,0x6e,0x67,0x20,0x6f,0x66,0x20,0x6c,0x6f,0x61,0x64,0x69,0x6e,0x67, 0 ; "TODO: tokenizing of loading"
	str_30: db 0x54,0x4f,0x44,0x4f,0x3a,0x20,0x74,0x6f,0x6b,0x65,0x6e,0x69,0x7a,0x69,0x6e,0x67,0x20,0x6f,0x66,0x20,0x73,0x74,0x6f,0x72,0x69,0x6e,0x67, 0 ; "TODO: tokenizing of storing"
