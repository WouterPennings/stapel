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
	; --- PushInt(3) ---
	mov rax, 3
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
	; --- Put ---
	pop rdi
	call print_i64
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- PushInt(7) ---
	mov rax, 7
	push rax
	; --- InfixOperator(GreaterThan) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	setg cl
	push rcx
	; --- Put ---
	pop rdi
	call print_i64
	; --- PushInt(3) ---
	mov rax, 3
	push rax
	; --- If ---
	; --- Dup ---
	pop rax
	push rax
	push rax
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- InfixOperator(LesserThan) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	setl cl
	push rcx
	; --- Over ---
	pop rax
	pop rbx
	push rbx
	push rax
	push rbx
	; --- PushInt(7) ---
	mov rax, 7
	push rax
	; --- InfixOperator(GreaterThan) ---
	pop rbx
	pop rax
	xor rcx, rcx
	cmp rax, rbx
	setg cl
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
	je .addr_2
	; --- PushInt(1) ---
	mov rax, 1
	push rax
	; --- Put ---
	pop rdi
	call print_i64
	jmp .addr_1
.addr_2:
	; --- PushInt(0) ---
	mov rax, 0
	push rax
	; --- Put ---
	pop rdi
	call print_i64
.addr_1:
	; === GLOBAL EXIT ===
	mov rax, 60
	mov rdi, 0
	syscall


section .bss

section .data
; Strings with null terminators
