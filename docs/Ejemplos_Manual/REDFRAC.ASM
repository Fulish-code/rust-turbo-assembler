;**********************************************************
; REDFRAC.ASM
;
; Este programa reduce la fracción numer/denom a su mínima
; expresión usando el procedimiento REDFRAC del capítulo 6
; (variables locales en la pila). Por ejemplo 12/60 se
; reduce a 1/5.
;
;** El programa principal no viene en el manual: se agregó para
;** poder ensamblar, ejecutar y depurar (TD) el procedimiento.
;
; El pseudocódigo de este programa es:
;
;   AX = numer
;   DX = denom
;   redfrac()
;   numer = AX
;   denom = DX
;**********************************************************

;****** CÓDIGO DE INICIO **********************************

        ideal
        dosseg
        model   small
        stack   256

;****** VARIABLES DEL PROGRAMA ****************************

        dataseg
codsal  db      0
numer   dw      12
denom   dw      60

;****** CÓDIGO DEL PROGRAMA *******************************

        codeseg
inicio:
        mov     ax, @data               ; Inicializa el
        mov     ds, ax                  ; segmento de datos

        mov     ax, [numer]             ; AX = numer
        mov     dx, [denom]             ; DX = denom

        call    redfrac                 ; redfrac()

        mov     [numer], ax             ; numer = AX
        mov     [denom], dx             ; denom = DX

salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** PROCEDIMIENTOS ************************************

;**********************************************************
; REDFRAC
;
; Este procedimiento reduce una fracción a su mínima
; expresión
;
; Parámetros:
;
;       AX = numerador
;       DX = denominador
;
; Regresa:
;
;       AX = numerador reducido
;       DX = denominador reducido
;
; El pseudocódigo para este procedimiento es:
;

;   BX = mcd(numerador, denominador)
;
;   numerador /= BX
;   denominador /= BX
;
;*********************************************************
proc    redfrac
        push    bp                      ; Preserva BP
        mov     bp, sp                  ; Crea variables
        sub     sp, 4                   ; locales en la pila
        push    bx                      ; Preserva BX

;num = [BP-2], den = [BP-4]
        mov     [bp-2], ax              ; num = numerador
        mov     [bp-4], dx              ; den = denominador

        call    mcd                     ; BX = mcd(numerador,
                                        ;          denominador)
        mov     bx, ax
        xor     dx, dx                  ; num = numerador/BX
        mov     ax, [bp-2]
        div     bx
        mov     [bp-2], ax

        mov     ax, [bp-4]              ; DX = denominador/BX
        div     bx
        mov     dx, ax
        mov     ax, [bp-2]              ; AX = num

        pop     bx                      ; Restaura BX
        mov     sp, bp                  ; Elimina variables locales
        pop     bp                      ; Restaura BP
        ret
endp    redfrac

;**********************************************************
; MCD
;
;** No viene en el manual: es el ejercicio 1 sobre
;** procedimientos (capítulo 6), que REDFRAC necesita.
;
; Este procedimiento obtiene el máximo común divisor de dos
; números no signados de tipo palabra (algoritmo de Euclides).
;
; Parámetros:
;
;       AX = primer número
;       DX = segundo número
;
; Regresa:
;
;       AX = mcd(AX, DX)
;
; El pseudocódigo de este procedimiento es:
;
;   while(DX != 0)
;   {
;      BX = DX
;      DX = AX % BX
;      AX = BX
;   }
;**********************************************************
proc    mcd
        push    bx                      ; Preserva BX, DX
        push    dx

@@whi:  cmp     dx, 0                   ; while(DX != 0)
        je      @@endwhi                ; {
        mov     bx, dx                  ;    BX = DX
        xor     dx, dx                  ;    DX = AX % BX
        div     bx
        mov     ax, bx                  ;    AX = BX
        jmp     @@whi                   ; }
@@endwhi:
        pop     dx                      ; Restaura DX, BX
        pop     bx
        ret
endp    mcd

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
