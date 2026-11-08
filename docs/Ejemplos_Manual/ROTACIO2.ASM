;**********************************************************
; ROTACIO2.ASM
;

; Este programa ejecuta todas las rotaciones de un bit con
; una variable de tipo palabra doble. Las operaciones de
; rotación se implementan mediante macros que rotan el
; contenido de la localidad de memoria dada por el
; parámetro de la macro un bit.
;**********************************************************

;****** CÓDIGO DE INICIO **********************************

        ideal
        dosseg
        model   small
        stack   256

;****** MACROS ********************************************

;**********************************************************
; RCLDW1
;
; Esta macro rota el contenido de dest un bit a la
; izquierda a través de la bandera de acarreo.
;**********************************************************
macro   rcldw1 dest
        rcl     [word dest], 1          ;; rcl dest, 1
        rcl     [word dest+2], 1
endm    rcldw1

;**********************************************************
; RCRDW1
;
; Esta macro rota el contenido de dest un bit a la
; derecha a través de la bandera de acarreo.
;**********************************************************
macro   rcrdw1 dest
        rcr     [word dest+2], 1        ;; rcr dest, 1
        rcr     [word dest], 1
endm    rcrdw1

;**********************************************************
; ROLDW1
;
; Esta macro rota el contenido de dest un bit a la izquierda
;**********************************************************
macro   roldw1 dest
        push    bx                      ;; Preserva BX
        mov     bx, [word dest+2]       ;; rol dest, 1
        shl     bx, 1
        rcl     [word dest], 1
        rcl     [word dest+2], 1
        pop     bx                      ;; Restaura BX
endm    roldw1

;**********************************************************
; RORDW1
;
; Esta macro rota el contenido de dest un bit a la derecha.
;**********************************************************
macro   rordw1 dest
        push    bx                      ;; Preserva BX
        mov     bx, [word dest]         ;; ror dest, 1
        shr     bx, 1
        rcr     [word dest+2], 1
        rcr     [word dest], 1
        pop     bx                      ;; Restaura BX
endm    rordw1

;*** VARIABLES DEL PROGRAMA *******************************

        dataseg
codsal  db      0
r_rcl   dd      ?
r_rcr   dd      ?
r_rol   dd      ?
r_ror   dd      ?

;****** CÓDIGO DEL PROGRAMA *******************************

        codeseg
inicio:
        mov     ax, @data               ; Inicializa el
        mov     ds, ax                  ; segmento de datos

; Rotación a la izquierda a través de la bandera de acarreo.

        rcldw1  r_rcl                   ; rcl r_rcl, 1

; Rotación a la derecha a través de la bandera de acarreo
        rcrdw1  r_rcr                   ; rcr r_rcr, 1

; Rotación a la izquierda
        roldw1  r_rol                   ; rol r_rol, 1

; Rotación a la derecha
        rordw1  r_ror                   ; ror r_ror, 1

salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
