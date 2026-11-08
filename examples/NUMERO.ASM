;**********************************************************
; NUMERO.ASM
;
; Este programa despliega en decimal un número de tipo
; palabra no signado. Utiliza un procedimiento que guarda
; los dígitos en la pila para desplegarlos en orden.
;**********************************************************

;****** CÓDIGO DE INICIO **********************************

        ideal
        dosseg
        model   small
        stack   256

;****** VARIABLES DEL PROGRAMA ****************************

        dataseg
codsal  db      0
texto   db      'El valor es: $'
valor   dw      31416

;****** CÓDIGO DEL PROGRAMA *******************************

        codeseg
inicio:
        mov     ax, @data               ; Inicializa el
        mov     ds, ax                  ; segmento de datos

        mov     dx, offset texto
        mov     ah, 09h
        int     21h

        mov     ax, [valor]             ; AX = valor
        call    pdecimal                ; pdecimal()

salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** PROCEDIMIENTOS ************************************

;**********************************************************
; PDECIMAL
;
; Este procedimiento despliega en decimal un número de tipo
; palabra no signado.
;
; Parámetro:
;
;       AX = número
;
; Regresa:
;
;       Nada
;
; El pseudocódigo de este procedimiento es:
;
;   n = 0
;   do
;   {
;      push(AX % 10)
;      AX /= 10
;      n++
;   }
;   while(AX != 0)
;
;   do
;   {
;      despliega pop() + '0'
;   }
;   while(--n > 0)
;**********************************************************
proc    pdecimal
        push    ax                      ; Preserva AX, BX, CX, DX
        push    bx
        push    cx
        push    dx

        mov     bx, 10
        xor     cx, cx                  ; n = 0
@@divide:                               ; do
                                        ; {
        xor     dx, dx                  ;    DX = AX % 10
        div     bx                      ;    AX /= 10
        push    dx                      ;    push(DX)
        inc     cx                      ;    n++
        test    ax, ax                  ; }
        jnz     @@divide                ; while(AX != 0)

@@saca:                                 ; do
                                        ; {
        pop     dx                      ;    DL = pop() + '0'
        add     dl, '0'
        mov     ah, 02h                 ;    Despliega DL
        int     21h
        loop    @@saca                  ; }
                                        ; while(--n > 0)

        pop     dx                      ; Restaura DX, CX, BX, AX
        pop     cx
        pop     bx
        pop     ax
        ret
endp    pdecimal

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
