;**********************************************************
; ROTA_N.INC
;
; Este archivo contiene una serie de macros que pueden
; incluirse en otros programas con la directiva include e
; implementan las operaciones de rotación del contenido de
; la localidad de memoria dada por el parámetro dest un
; número de bits dado por el parámetro cnta.
;**********************************************************

;**********************************************************
; RCLDWN
;
; Esta macro rota el contenido de dest cnta bits a la
; izquierda a través de la bandera de acarreo.
;**********************************************************
macro   rcldwn dest, cnta
        rept    cnta
            rcl     [word dest], 1      ;; rcl dest, 1
            rcl     [word dest+2], 1
        endm
endm    rcldwn

;**********************************************************
; RCRDWN
;
; Esta macro rota el contenido de dest cnta bits a la
; derecha a través de la bandera de acarreo.
;**********************************************************
macro   rcrdwn dest, cnta
        rept    cnta
            rcr     [word dest+2], 1    ;; rcr dest, 1
            rcr     [word dest], 1
        endm
endm    rcrdwn
;**********************************************************
; ROLDWN
;
; Esta macro rota el contenido de dest cnta bits a la
; izquierda.
;**********************************************************
macro   roldwn dest, cnta
        push    bx                      ;; Preserva BX
        rept    cnta
            mov     bx, [word dest+2]   ;; rol dest, 1
            shl     bx, 1
            rcl     [word dest], 1
            rcl     [word dest+2], 1
        endm
        pop     bx                      ;; Restaura BX
endm    roldwn

;**********************************************************
; RORDWN
;
; Esta macro rota el contenido de dest cnta bits a la
; derecha.
;**********************************************************
macro   rordwn dest, cnta
        push    bx                      ;; Preserva BX
        rept    cnta
            mov     bx, [word dest]     ;; ror dest, 1
            shr     bx, 1
            rcr     [word dest+2], 1
            rcr     [word dest], 1
        endm
        pop     bx                      ;; Restaura BX
endm    rordwn
