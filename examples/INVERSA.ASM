;**********************************************************
; INVERSA.ASM
;
; Este programa lee una cadena del teclado (función 0Ah
; de la interrupción 21h) y la despliega al revés.
;**********************************************************

;****** CÓDIGO DE INICIO **********************************

        ideal
        dosseg
        model   small
        stack   256

;****** DECLARACIÓN DE CONSTANTES SIMBÓLICAS **************

TAMMAX  equ     60

;****** VARIABLES DEL PROGRAMA ****************************

        dataseg
codsal  db      0
pide    db      'Escribe algo: $'
salto   db      13, 10, 'Al reves: $'
bufer   db      TAMMAX                  ; Tamaño máximo
nleidos db      ?                       ; Caracteres leídos
texto   db      TAMMAX dup(?)           ; Texto

;****** CÓDIGO DEL PROGRAMA *******************************

        codeseg
inicio:
        mov     ax, @data               ; Inicializa el
        mov     ds, ax                  ; segmento de datos

        mov     dx, offset pide         ; Pide la cadena
        mov     ah, 09h
        int     21h

        mov     dx, offset bufer        ; Lee la cadena
        mov     ah, 0Ah
        int     21h

        mov     dx, offset salto
        mov     ah, 09h
        int     21h

        xor     cx, cx                  ; CX = nleidos
        mov     cl, [nleidos]
        jcxz    salir                   ; if(CX == 0) goto salir
        mov     si, offset texto        ; SI = texto + CX - 1
        add     si, cx
        dec     si
do:                                     ; do
                                        ; {
        mov     dl, [si]                ;    Despliega [SI]
        mov     ah, 02h
        int     21h
        dec     si                      ;    SI--
        loop    do                      ; }
                                        ; while(--CX > 0)

salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
