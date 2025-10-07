
section .text
global our_code_starts_here
our_code_starts_here:
  
mov rax, 1
mov [rsp - 16], rax
mov rax, 10
mov [rsp - 24], rax

mov rax, 100
mov [rsp - 40], rax
mov rax, [rsp - 16]
mov [rsp - 56], rax 
mov rax, [rsp - 24]
mov [rsp - 64], rax 
mov rax, [rsp - 40]
add rax, [rsp - 64]
add rax, [rsp - 56]
  ret
