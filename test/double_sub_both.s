
section .text
global our_code_starts_here
our_code_starts_here:
  mov rax, 10
mov [rsp - 24], rax 
mov rax, 50
sub rax, [rsp - 24]
mov [rsp - 16], rax 
mov rax, 10
mov [rsp - 16], rax 
mov rax, 60
sub rax, [rsp - 16]
sub rax, [rsp - 16]
  ret
