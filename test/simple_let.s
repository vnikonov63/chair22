
section .text
global our_code_starts_here
our_code_starts_here:
  
mov rax, 11
mov [rsp - 16], rax
mov rax, 1
mov [rsp - 24], rax
mov rax, [rsp - 24]
mov [rsp - 40], rax 
mov rax, [rsp - 16]
sub rax, [rsp - 40]
  ret
