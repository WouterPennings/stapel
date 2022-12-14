; X86_64 Assembly linux (Ubuntu 22.04)
; NASM version 2.15.05
global _start

section .data
stack_size DD 0

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

_inc_stack_count:
    mov rax, [stack_size]
	inc rax
    mov [stack_size], rax
    ret

_dec_stack_count:
    mov rax, [stack_size]
	sub rax, 1
    mov [stack_size], rax
    ret

_start: