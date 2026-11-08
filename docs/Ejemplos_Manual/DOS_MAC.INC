;*************************************************************
; DOS_MAC.INC
;
; Este archivo contiene macros que permiten desplegar un
; carácter a pantalla y leer un carácter del teclado. Las
; macros utilizan llamadas a las rutinas del DOS
;*************************************************************

;*************************************************************
; APUTCHAR
;
; Esta macro despliega un carácter en la pantalla
;
; Parámetro:
;
;       DL = carácter
;
; Regresa:
;
;       Nada
;*************************************************************

macro   aputchar
        mov     ah, 02h                 ;; Despliega un carácter
        int     21h                     ;; Interrupción del DOS
endm    aputchar

;*************************************************************
; AGETCHAR
;
; Esta macro lee un carácter del teclado
;
; Parámetro:
;
;       Nada
;
; Regresa:
;
;       AL = Código ASCII del carácter
;*************************************************************
macro   agetchar
        mov     ah, 01h                 ;; Lee un carácter
        int     21h                     ;; Interrupción del DOS
endm    agetchar
