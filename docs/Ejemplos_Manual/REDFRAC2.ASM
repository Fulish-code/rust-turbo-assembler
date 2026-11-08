;**********************************************************
; REDFRAC2.ASM
;
; Este programa reduce la fracción numer/denom a su mínima
; expresión usando el procedimiento REDFRAC2 (directiva local) del capítulo 6
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
; REDFRAC2
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
;   numerador /= BX
;   denominador /= BX
;**********************************************************
proc    redfrac
; Declara las variables locales en la pila
        local   num: word, den: word = tamVarsLoc

        push    bp                      ; Preserva BP
        mov     bp, sp                  ; Crea variables
        sub     sp, tamVarsLoc          ; locales en la pila
        push    bx                      ; Preserva BX

        mov     [num], ax               ; num = numerador
        mov     [den], dx               ; den = denominador

        call    mcd                     ; BX = mcd(numerador,
        mov     bx, ax                  ;          denominador)

        xor     dx, dx                  ; num = numerador/BX
        mov     ax, [num]
        div     bx
        mov     [num], ax

        mov     ax, [den]               ; DX = denominador/BX
        div     bx

        mov     dx, ax
        mov     ax, [num]               ; AX = num

        pop     bx                      ; Restaura BX
        mov     sp, bp                  ; Elimina variables
                                        ; locales

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
