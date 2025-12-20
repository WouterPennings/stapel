; X86_64 Assembly linux (Ubuntu 22.04)
; NASM version 2.15.05
global _start

section .text
; The print functions is written in C and compiled to assembly. (https://godbolt.org, -O3)
; did need some tweaking after. Source: https://github.com/wouterpennings/print_c
; Can print postive and negative integers
print_i32:
    sub     rsp, 40
    xor     r9, r9
    test    edi, edi
    jns     .L2
    neg     edi
    mov     r9, 1
.L2:
    mov     BYTE[rsp+31], 10
    lea     rsi, [rsp+30]
    mov     edx, 1
    mov     r8, 3435973837
.L3:
    mov     eax, edi
    sub     rsi, 1
    imul    rax, r8
    shr     rax, 35
    lea     ecx, [rax+rax*4]
    add     ecx, ecx
    sub     edi, ecx
    mov     ecx, edx
    add     edx, 1
    add     edi, 48
    mov     BYTE [rsi+1], dil
    mov     edi, eax
    test    eax, eax
    jne     .L3
    test    r9, r9
    je      .L4
    movsx   rdx, edx
    not     rdx
    mov     BYTE [rsp+32+rdx], 45
    lea     edx, [rcx+2]
.L4:
    movsx   rdx, edx
    mov     eax, 32
    mov     edi, 1
    sub     rax, rdx
    lea     rsi, [rsp+rax]
    mov     rax, 1
    syscall
    add     rsp, 40
    ret

proc_interceptor:
    pop rax ; Return address
    pop rdi ; Procedure adress

    ; Pushing return address to return stack
    mov rbx, [ret_stack_cursor]
    inc rbx
    mov [ret_stack_cursor], rbx
    mov [ret_stack+rbx*8], rax

    ; jumping to procedure
    jmp rdi

_start:
    ; DEFAULT INSTRUCTIONS
    mov [ori_stack_ptr], esp

    jmp .proc_main
    ; ADDED COMPILED INSTRUCTIONS

	; === proc main do ===
.proc_main:
	; === PushInt(0) === (6d6501bd-12ca-4e4a-afcf-9c7da27be4d9)
	push 0
	; === While === (25575c67-2945-4082-b783-dc2ed83ceb05)
.addr_84629611788806980053722598304359692224:
	; === Dup === (3fab1202-a0a6-4a39-bd76-e289f8dcb7c0)
	pop rax
	push rax
	push rax
	; === PushInt(1000000) === (3dc116f5-b82e-405a-94a2-00f5801a74b5)
	push 1000000
	; === InfixOperator(NotEquals) === (36128fff-af2e-498b-9564-8c1d2736c636)
	pop rax
	pop rbx
	mov rcx, 0
	mov rdx, 1
	cmp rax, rbx
	cmovne rcx, rdx
	push rcx
	; === end ===
	cmp rax, 0
	je .addr_5273225451182422158764835158470319861
	; === PrefixOperator(Plus) === (03f79616-7217-4aac-8a8b-5cadf5d25ef5)
	pop rax
	inc rax
	push rax
	; === end ===
	jmp .addr_84629611788806980053722598304359692224
.addr_5273225451182422158764835158470319861:
	; === Put === (1b26b9ee-d7d4-4645-826e-39c17f13a8f0)
	pop rax
	mov rdi, rax
	call print_i32
	; === end ===
	; === END OF PROGRAM (ADDED DURING COMPILATION) ===
	mov rax, 60
	mov rdi, 0
	syscall

section .bss
mem: resq 1024 ; Pointer to start of memory

section .data
ori_stack_ptr: dd 0 ; Pointer to start of stack
ret_stack: TIMES 1024 DQ 0; Stack for the return adresses
ret_stack_cursor: DQ 0; Pointer to start of memory
; Strings defined by user in the program
